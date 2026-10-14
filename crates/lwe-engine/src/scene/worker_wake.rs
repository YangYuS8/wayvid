//! A result-ready hint belongs to one renderer, even when generations share a ping.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

use calloop::ping::Ping;

struct Ready {
    origin: Instant,
    // Zero means no hint; otherwise the first coalesced publication's elapsed
    // microseconds plus one. One atomic keeps readiness and timing inseparable.
    published: AtomicU64,
}

#[derive(Clone)]
pub(crate) struct WorkerWake {
    ready: Arc<Ready>,
    ping: Ping,
}

impl WorkerWake {
    pub(crate) fn new(ping: Ping) -> Self {
        Self {
            ready: Arc::new(Ready {
                origin: Instant::now(),
                published: AtomicU64::new(0),
            }),
            ping,
        }
    }

    /// Call only after publishing the output/error and releasing its mailbox.
    pub(crate) fn notify(&self) {
        let micros = u64::try_from(self.ready.origin.elapsed().as_micros())
            .unwrap_or(u64::MAX - 1)
            .min(u64::MAX - 1);
        let _ = self.ready.published.compare_exchange(
            0,
            micros + 1,
            Ordering::Release,
            Ordering::Relaxed,
        );
        self.ping.ping();
    }

    /// Publication-to-event-loop dispatch latency, not packet consumption or GPU time.
    pub(crate) fn take(&self) -> Option<Duration> {
        let published = self.ready.published.swap(0, Ordering::Acquire);
        (published != 0).then(|| {
            self.ready
                .origin
                .elapsed()
                .saturating_sub(Duration::from_micros(published - 1))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calloop::{EventLoop, ping::make_ping};

    #[test]
    fn coalesced_and_late_generation_pings_cannot_mark_another_owner_ready() {
        let (ping, source) = make_ping().unwrap();
        let previous = WorkerWake::new(ping.clone());
        let current = WorkerWake::new(ping);
        let mut event_loop = EventLoop::<usize>::try_new().unwrap();
        let active = current.clone();
        event_loop
            .handle()
            .insert_source(source, move |(), _, consumed| {
                if active.take().is_some() {
                    *consumed += 1;
                }
            })
            .unwrap();
        let mut consumed = 0;
        previous.notify();
        event_loop.dispatch(Duration::ZERO, &mut consumed).unwrap();
        assert_eq!(consumed, 0);
        assert!(previous.take().is_some());
        current.notify();
        current.notify();
        event_loop.dispatch(Duration::ZERO, &mut consumed).unwrap();
        assert_eq!(consumed, 1);
        assert!(current.take().is_none());
        current.notify();
        event_loop.dispatch(Duration::ZERO, &mut consumed).unwrap();
        assert_eq!(consumed, 2);
    }
}
