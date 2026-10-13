//! Per-output deadlines are independent of how often commands wake calloop.

use std::time::{Duration, Instant};

use anyhow::{Result, ensure};

pub(super) const FRAME_POLL_INTERVAL: Duration = Duration::from_millis(16);

pub(super) fn frame_interval(fps: Option<u32>) -> Result<Option<Duration>> {
    fps.map(|fps| {
        ensure!(fps != 0, "Wallpaper FPS limit must be greater than zero");
        Ok(Duration::from_nanos(
            1_000_000_000_u64.div_ceil(u64::from(fps)),
        ))
    })
    .transpose()
}

#[derive(Debug, Default)]
pub(super) struct FrameSchedule {
    next_attempt: Option<Instant>,
    last_attempt: Option<(Instant, bool)>,
}

impl FrameSchedule {
    pub(super) fn ready(&self, now: Instant) -> bool {
        self.next_attempt.is_none_or(|deadline| now >= deadline)
    }

    pub(super) fn delay(&self, now: Instant) -> Duration {
        self.next_attempt
            .map(|deadline| deadline.saturating_duration_since(now))
            .unwrap_or_default()
    }

    /// New sources, configure, resume and rollback may present immediately.
    pub(super) fn wake(&mut self) {
        self.next_attempt = None;
        self.last_attempt = None;
    }

    pub(super) fn set_interval(&mut self, interval: Option<Duration>) {
        if let Some((last, presented)) = self.last_attempt {
            self.attempted(last, presented, interval);
        }
    }

    pub(super) fn attempted(&mut self, now: Instant, presented: bool, interval: Option<Duration>) {
        self.last_attempt = Some((now, presented));
        // A decoder without a frame must be checked again without manufacturing
        // Wayland frame callbacks. Its polling cannot exceed the configured FPS.
        let wait = if presented {
            interval.unwrap_or_default()
        } else {
            interval.unwrap_or(FRAME_POLL_INTERVAL)
        };
        self.next_attempt = now.checked_add(wait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequent_command_and_compositor_wakes_do_not_bypass_fps_deadlines() {
        let start = Instant::now();
        let interval = frame_interval(Some(30)).unwrap();
        let mut schedule = FrameSchedule::default();
        let mut presentations = Vec::new();
        // Simulate an already-ready compositor with unrelated commands at 1kHz.
        for millisecond in 0..1000 {
            let now = start + Duration::from_millis(millisecond);
            if schedule.ready(now) {
                presentations.push(now);
                schedule.attempted(now, true, interval);
            }
        }
        assert_eq!(presentations.len(), 30);
        for frames in presentations.windows(2) {
            assert!(frames[1] - frames[0] >= interval.unwrap());
        }
    }

    #[test]
    fn slow_swap_does_not_create_a_catch_up_burst() {
        let start = Instant::now();
        let mut schedule = FrameSchedule::default();
        let interval = frame_interval(Some(60)).unwrap();
        let swap_finished = start + Duration::from_secs(2);
        schedule.attempted(swap_finished, true, interval);
        assert!(!schedule.ready(swap_finished));
        assert!(schedule.ready(swap_finished + interval.unwrap()));
    }

    #[test]
    fn forced_replacement_or_rollback_wakes_a_throttled_output() {
        let now = Instant::now();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(now, true, frame_interval(Some(1)).unwrap());
        assert!(!schedule.ready(now + FRAME_POLL_INTERVAL));
        schedule.wake();
        assert!(schedule.ready(now + FRAME_POLL_INTERVAL));
    }

    #[test]
    fn empty_decoder_attempts_have_a_deadline_even_without_a_fps_limit() {
        let now = Instant::now();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(now, false, None);
        assert!(!schedule.ready(now));
        assert_eq!(schedule.delay(now), FRAME_POLL_INTERVAL);
        assert!(schedule.ready(now + FRAME_POLL_INTERVAL));
    }

    #[test]
    fn zero_fps_is_rejected_and_fractional_frame_intervals_round_up() {
        assert!(frame_interval(Some(0)).is_err());
        assert_eq!(frame_interval(None).unwrap(), None);
        assert_eq!(
            frame_interval(Some(30)).unwrap().unwrap().as_nanos(),
            33_333_334
        );
        assert!(!frame_interval(Some(u32::MAX)).unwrap().unwrap().is_zero());
    }

    #[test]
    fn live_limit_changes_keep_the_last_swap_as_the_deadline_origin() {
        let start = Instant::now();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(start, true, frame_interval(Some(60)).unwrap());
        schedule.set_interval(frame_interval(Some(15)).unwrap());
        assert!(!schedule.ready(start + Duration::from_millis(20)));
        assert!(schedule.ready(start + Duration::from_millis(67)));
        schedule.set_interval(frame_interval(Some(120)).unwrap());
        assert!(schedule.ready(start + Duration::from_millis(9)));
    }
}
