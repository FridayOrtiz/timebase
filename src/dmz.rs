use std::convert::{TryFrom, TryInto};
use std::error::Error;

use aya::maps::{MapRefMut, PerfEventArray};
use aya::programs::{tc, Link, SchedClassifier, TcAttachType};
use aya::util::online_cpus;
use aya::Bpf;
use slog::{debug, warn};

use crate::{perf::poll_buffers, LOGGER};

pub fn run_dmz(interface_name: &str) -> Result<(), Box<dyn Error>> {
    let mut bpf = Bpf::load_file("bpf/filter_program_x86_64")?;
    if let Err(e) = tc::qdisc_add_clsact(interface_name) {
        warn!(LOGGER, "Interface already configured: {:?}", e);
        warn!(
            LOGGER,
            "If the filter misbehaves, manually remove the tc qdisc."
        );
        warn!(LOGGER, "You can probably ignore this.");
    }

    let prog: &mut SchedClassifier = bpf.program_mut("ntp_filter")?.try_into()?;
    prog.load()?;
    let mut egress_linkref = prog.attach(interface_name, TcAttachType::Egress)?;
    debug!(LOGGER, "NTP outbound filter loaded and attached.");

    let prog: &mut SchedClassifier = bpf.program_mut("ntp_receiver")?.try_into()?;
    prog.load()?;
    let mut ingress_linkref = prog.attach(interface_name, TcAttachType::Ingress)?;
    debug!(LOGGER, "NTP inbound filter loaded and attached.");

    // now we need to initialize the ringbuffer indices for the DMZ
    let mut msg_ctr = aya::maps::Array::<MapRefMut, u32>::try_from(bpf.map_mut("msg_ctr")?)?;
    msg_ctr
        .set(0, 0, 0)
        .expect("could not set bottom of buffer");
    msg_ctr.set(1, 1, 0).expect("could not set top of buffer");

    let mut perf_array = PerfEventArray::try_from(bpf.map_mut("ntp_filter_events")?)?;

    let mut perf_buffers = Vec::new();
    for cpuid in online_cpus()? {
        perf_buffers.push(perf_array.open(cpuid, None)?);
    }

    // poll the buffers to know when they have queued events
    poll_buffers(perf_buffers);

    ingress_linkref.detach()?;
    egress_linkref.detach()?;

    debug!(LOGGER, "NTP filter detached.");

    Ok(())
}
