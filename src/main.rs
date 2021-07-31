use std::fmt::{Display, Formatter};

use clap::{crate_authors, crate_description, crate_version, App, Arg, SubCommand};
use lazy_static::lazy_static;
use slog::{debug, o, Drain, Logger};
use slog_term::TermDecorator;

mod client;
mod dmz;
mod server;

const ETHERNET_HEADER_LEN: usize = 14;
const IPV4_HEADER_LEN: usize = 20;
const UDP_HEADER_LEN: usize = 8;
const VALUE_LEN: usize = 99;

#[repr(C)]
#[derive(Copy, Clone)]
struct DataChunk {
    data: [u8; VALUE_LEN],
}

unsafe impl aya::Pod for DataChunk {}

lazy_static! {
    pub static ref LOGGER: Logger = Logger::root(
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
        write!(f, "{}", hex::encode(self.value))
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
                ),
        )
        .subcommand(
            SubCommand::with_name("dmz")
                .about("send information to the server")
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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("server") {
        debug!(LOGGER, "Starting timebase server.");
        let interface = matches.value_of("interface").unwrap();
        server::load_filter(interface).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("client") {
        let interface = matches.value_of("interface").unwrap();
        client::run_client(interface);
    } else if let Some(matches) = matches.subcommand_matches("dmz") {
        let interface = matches.value_of("interface").unwrap();
        dmz::run_dmz(interface);
    } else {
        println!("Please specify `client` or `server`.");
    }
}
