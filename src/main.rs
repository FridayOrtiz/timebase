use aya::maps::perf::PerfEventArrayBuffer;
use aya::maps::{MapRefMut, PerfEventArray};
use aya::programs::{tc, Link, SchedClassifier, TcAttachType};
use aya::util::online_cpus;
use aya::Bpf;
use byteorder::{LittleEndian, ReadBytesExt};
use bytes::BytesMut;
use clap::{crate_authors, crate_description, crate_version, App, Arg, SubCommand};
use lazy_static::lazy_static;
use mio::unix::SourceFd;
use mio::{Events, Interest, Token};
use pnet::datalink::{Channel, NetworkInterface};
use slog::{crit, debug, o, warn, Drain, Logger};
use slog_term::TermDecorator;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::os::unix::io::AsRawFd;
use std::time::Duration;

const ETHERNET_HEADER_LEN: usize = 14;
const IPV4_HEADER_LEN: usize = 20;
const UDP_HEADER_LEN: usize = 8;
const VALUE_LEN: usize = 8;

lazy_static! {
    static ref LOGGER: Logger = Logger::root(
        slog_async::Async::new(
            slog_term::FullFormat::new(TermDecorator::new().build())
                .build()
                .fuse(),
        )
        .build()
        .fuse(),
        o!()
    );
}

#[repr(C)]
struct NtpExtensionless {
    lvm: u8,
    stratum: u8,
    poll: u8,
    precision: u8,
    root_delay: u32,
    root_dispersion: u32,
    reference_id: u32,
    reference_ts: u64,
    originate_ts: u64,
    receive_ts: u64,
    transmit_ts: u64,
}

#[repr(C, align(4))]
struct ExtensionField {
    field_type: u16,
    field_len: u16,
    value: [u8; VALUE_LEN],
}

impl Display for ExtensionField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = String::from_utf8(Vec::from(self.value)).map_err(|_| std::fmt::Error)?;
        let value = value.trim_matches('\0');
        write!(f, "{}", value)
    }
}

fn poll_buffers(buf: Vec<PerfEventArrayBuffer<MapRefMut>>) {
    let mut poll = mio::Poll::new().unwrap();

    let mut out_bufs = [BytesMut::with_capacity(1024)];

    let mut tokens: HashMap<Token, PerfEventArrayBuffer<MapRefMut>> = buf
        .into_iter()
        .map(
            |p| -> Result<(Token, PerfEventArrayBuffer<MapRefMut>), Box<dyn Error>> {
                let token = Token(p.as_raw_fd() as usize);
                poll.registry().register(
                    &mut SourceFd(&p.as_raw_fd()),
                    token,
                    Interest::READABLE,
                )?;
                Ok((token, p))
            },
        )
        .collect::<Result<HashMap<Token, PerfEventArrayBuffer<MapRefMut>>, Box<dyn Error>>>()
        .unwrap();

    let mut events = Events::with_capacity(1024);
    loop {
        match poll.poll(&mut events, Some(Duration::from_millis(100))) {
            Ok(_) => {
                let token_list: Vec<Token> = events
                    .iter()
                    .filter(|event| event.is_readable())
                    .map(|e| e.token())
                    .collect();
                token_list.into_iter().for_each(|t| {
                    let buf = tokens.get_mut(&t).unwrap();
                    buf.read_events(&mut out_bufs).unwrap();
                    debug!(LOGGER, "buf: {:?}", out_bufs.get(0).unwrap());
                });
            }
            Err(e) => {
                crit!(LOGGER, "critical error: {:?}", e);
                panic!()
            }
        }
    }
}

fn load_filter(interface_name: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut bpf = Bpf::load_file("bpf/filter_program_x86_64")?;
    if let Err(e) = tc::qdisc_add_clsact(interface_name) {
        warn!(LOGGER, "Interface already configured: {:?}", e);
        warn!(
            LOGGER,
            "If the filter misbehaves, manually remove the tc qdisc."
        );
        warn!(LOGGER, "You can probably ignore this.");
    }

    debug!(LOGGER, "Writing '{}' to map.", message);
    let mut msg_array = aya::maps::Array::<MapRefMut, u64>::try_from(bpf.map_mut("msg_array")?)?;
    let mut idx = 0;
    message
        .as_bytes()
        .chunks(VALUE_LEN)
        .into_iter()
        .for_each(|ch| {
            let mut ch = ch.to_vec();
            for _ in ch.len()..8 {
                ch.extend_from_slice(&[0u8]);
            }
            let ch = ch.as_slice().read_u64::<LittleEndian>().unwrap();
            msg_array.set(idx, ch, 0).expect("could not write to map");
            idx += 1;
        });

    let prog: &mut SchedClassifier = bpf.program_mut("ntp_filter")?.try_into()?;
    prog.load()?;
    let mut linkref = prog.attach(interface_name, TcAttachType::Egress)?;
    debug!(LOGGER, "NTP filter loaded and attached.");

    let mut perf_array = PerfEventArray::try_from(bpf.map_mut("ntp_filter_events")?)?;

    let mut perf_buffers = Vec::new();
    for cpuid in online_cpus()? {
        perf_buffers.push(perf_array.open(cpuid, None)?);
    }

    // poll the buffers to know when they have queued events
    poll_buffers(perf_buffers);

    linkref.detach()?;

    debug!(LOGGER, "NTP filter detached.");

    Ok(())
}

fn run_client(interface: &str) {
    let mut pnet_iface: NetworkInterface = NetworkInterface {
        name: "none".to_string(),
        description: "".to_string(),
        index: 0,
        mac: None,
        ips: vec![],
        flags: 0,
    };

    for iface in pnet::datalink::interfaces() {
        if iface.name.eq(interface) {
            pnet_iface = iface;
            break;
        }
    }

    if pnet_iface.name.eq("nonexistent") {
        panic!("could not find interface: {}", interface);
    }

    let (_tx, mut rx) = match pnet::datalink::channel(&pnet_iface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("unknown channel type"),
        Err(e) => panic!("err: {}", e),
    };

    println!("Listening on {}", pnet_iface.name);

    loop {
        let packet = rx.next().unwrap();
        let eth = pnet::packet::ethernet::EthernetPacket::new(packet).unwrap();
        if eth.get_ethertype() != pnet::packet::ethernet::EtherTypes::Ipv4 {
            continue;
        }
        let udp_packet = match pnet::packet::udp::UdpPacket::new(
            &packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)..],
        ) {
            Some(pkt) => pkt,
            None => continue,
        };

        if udp_packet.get_source() == 123 {
            let payload = &packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + UDP_HEADER_LEN)..];
            let extension: ExtensionField = unsafe {
                std::ptr::read(
                    payload[std::mem::size_of::<NtpExtensionless>()..].as_ptr() as *const _
                )
            };
            println!("value: {}", extension);
        }
    }
}

fn main() {
    let matches = App::new("Timebase")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("client")
                .about("receive information from the server")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .help("the interface to intercept and modify communications on")
                        .takes_value(true)
                        .required(true)
                        .value_name("INTERFACE NAME"),
                ),
        )
        .subcommand(
            SubCommand::with_name("server")
                .about("send information to the client")
                .arg(
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .help("the interface to intercept and modify communications on")
                        .takes_value(true)
                        .required(true)
                        .value_name("INTERFACE NAME"),
                )
                .arg(
                    Arg::with_name("message")
                        .short("m")
                        .long("message")
                        .help("the message to send")
                        .takes_value(true)
                        .required(true)
                        .value_name("'MESSAGE'"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        debug!(LOGGER, "Starting timebase server.");
        let interface = matches.value_of("interface").unwrap();
        let message = matches.value_of("message").unwrap();
        load_filter(interface, message).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("client") {
        let interface = matches.value_of("interface").unwrap();
        run_client(interface);
    } else {
        println!("Please specify `client` or `server`.");
    }
}
