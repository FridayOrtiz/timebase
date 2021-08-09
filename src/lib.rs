use lazy_static::lazy_static;
use slog::{o, Drain, Logger};
use slog_term::TermDecorator;
use std::fmt::{Display, Formatter};

pub mod client;
pub mod dmz;
pub mod perf;
pub mod server;

const ETHERNET_HEADER_LEN: usize = 14;
const IPV4_HEADER_LEN: usize = 20;
const UDP_HEADER_LEN: usize = 8;
const VALUE_LEN: usize = 91;

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
