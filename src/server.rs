use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

use crate::LOGGER;
use aya::maps::perf::PerfEventArrayBuffer;
use aya::maps::{MapRefMut, PerfEventArray};
use aya::programs::{tc, Link, SchedClassifier, TcAttachType};
use aya::util::online_cpus;
use aya::Bpf;
use bytes::BytesMut;
use mio::unix::SourceFd;
use mio::{Events, Interest, Token};
use slog::{crit, debug, warn};

use crate::{DataChunk, VALUE_LEN};

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

pub fn load_filter(interface_name: &str) -> Result<(), Box<dyn Error>> {
    let mut bpf = Bpf::load_file("bpf/filter_program_x86_64")?;
    let file_bytes = std::fs::read("bpf/filter_program_x86_64")?;
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
            ch.extend_from_slice(&[0u8]);
        }
        let ch = ch.as_slice(); //.read_u64::<LittleEndian>().unwrap();
        let mut data = [0u8; VALUE_LEN];
        data.copy_from_slice(ch);
        let d = DataChunk { data };
        msg_array.set(idx, d, 0).expect("could not write to map");
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
