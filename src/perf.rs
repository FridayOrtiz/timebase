use crate::LOGGER;
use aya::maps::perf::PerfEventArrayBuffer;
use aya::maps::MapRefMut;
use bytes::BytesMut;
use mio::unix::SourceFd;
use mio::{Events, Interest, Token};
use slog::{crit, debug};
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

pub(crate) fn poll_buffers(buf: Vec<PerfEventArrayBuffer<MapRefMut>>) {
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
