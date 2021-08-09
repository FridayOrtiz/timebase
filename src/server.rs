use std::convert::{TryFrom, TryInto};
use std::error::Error;

use crate::perf::poll_buffers;
use crate::LOGGER;
use aya::maps::{MapRefMut, PerfEventArray};
use aya::programs::{tc, Link, SchedClassifier, TcAttachType};
use aya::util::online_cpus;
use aya::Bpf;
use slog::{debug, warn};

use crate::{DataChunk, VALUE_LEN};

pub fn load_filter(interface_name: &str) -> Result<(), Box<dyn Error>> {
    let mut bpf = Bpf::load_file("bpf/filter_program_x86_64")?;
    let file_bytes = std::fs::read("lab/hello")?;
    if let Err(e) = tc::qdisc_add_clsact(interface_name) {
        warn!(LOGGER, "Interface already configured: {:?}", e);
        warn!(
            LOGGER,
            "If the filter misbehaves, manually remove the tc qdisc."
        );
        warn!(LOGGER, "You can probably ignore this.");
    }

    debug!(LOGGER, "Writing {} bytes to map.", file_bytes.len());
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
    debug!(LOGGER, "Writing {} chunks to map.", idx);
    // We repeat this section because the last two buffers will not be sent by the filter.
    // There's a logic flaw in my BPF ringbuffer implementation but adding more padding is just
    // easier.
    for _ in 0..3 {
        msg_array
            .set(
                idx,
                DataChunk {
                    data: [0xBEu8; VALUE_LEN],
                },
                0,
            )
            .expect("could not write done flag to map");
        idx += 1; // this represents the first _free_ slot in our ring buffer
    }

    // now we need to update the ringbuffer indices
    let mut msg_ctr = aya::maps::Array::<MapRefMut, u32>::try_from(bpf.map_mut("msg_ctr")?)?;
    msg_ctr
        .set(0, 0, 0)
        .expect("could not set bottom of buffer");
    msg_ctr.set(1, idx, 0).expect("could not set top of buffer");

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
