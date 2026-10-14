//! Per-output deadlines are independent of how often commands wake calloop.

use std::time::{Duration, Instant};

use anyhow::{Result, ensure};

pub(super) const FRAME_POLL_INTERVAL: Duration = Duration::from_millis(16);
const SCENE_PREPARATION_INTERVAL: Duration = Duration::from_millis(8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EmptyFrameRetry {
    Decoder,
    ScenePreparation,
}

impl EmptyFrameRetry {
    fn interval(self, presentation_interval: Option<Duration>) -> Duration {
        match self {
            Self::Decoder => presentation_interval.unwrap_or(FRAME_POLL_INTERVAL),
            Self::ScenePreparation => SCENE_PREPARATION_INTERVAL,
        }
    }
}

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
    last_presented: Option<Instant>,
    last_empty: Option<(Instant, EmptyFrameRetry)>,
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
        self.last_presented = None;
        self.last_empty = None;
    }

    pub(super) fn set_interval(&mut self, interval: Option<Duration>) {
        self.update_deadline(interval);
    }

    pub(super) fn attempted(
        &mut self,
        now: Instant,
        presented: bool,
        interval: Option<Duration>,
        retry: EmptyFrameRetry,
    ) {
        if presented {
            self.last_presented = Some(now);
            self.last_empty = None;
        } else {
            self.last_empty = Some((now, retry));
        }
        self.update_deadline(interval);
    }

    fn update_deadline(&mut self, interval: Option<Duration>) {
        // Preparation may be checked more often than presentations, but empty
        // attempts never replace the last successful swap as the FPS origin.
        let presentation = self
            .last_presented
            .and_then(|last| last.checked_add(interval.unwrap_or_default()));
        let retry = self
            .last_empty
            .and_then(|(last, retry)| last.checked_add(retry.interval(interval)));
        self.next_attempt = presentation.into_iter().chain(retry).max();
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
                schedule.attempted(now, true, interval, EmptyFrameRetry::Decoder);
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
        schedule.attempted(swap_finished, true, interval, EmptyFrameRetry::Decoder);
        assert!(!schedule.ready(swap_finished));
        assert!(schedule.ready(swap_finished + interval.unwrap()));
    }

    #[test]
    fn forced_replacement_or_rollback_wakes_a_throttled_output() {
        let now = Instant::now();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(
            now,
            true,
            frame_interval(Some(1)).unwrap(),
            EmptyFrameRetry::Decoder,
        );
        assert!(!schedule.ready(now + FRAME_POLL_INTERVAL));
        schedule.wake();
        assert!(schedule.ready(now + FRAME_POLL_INTERVAL));
    }

    #[test]
    fn empty_decoder_attempts_have_a_deadline_even_without_a_fps_limit() {
        let now = Instant::now();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(now, false, None, EmptyFrameRetry::Decoder);
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
        schedule.attempted(
            start,
            true,
            frame_interval(Some(60)).unwrap(),
            EmptyFrameRetry::Decoder,
        );
        schedule.set_interval(frame_interval(Some(15)).unwrap());
        assert!(!schedule.ready(start + Duration::from_millis(20)));
        assert!(schedule.ready(start + Duration::from_millis(67)));
        schedule.set_interval(frame_interval(Some(120)).unwrap());
        assert!(schedule.ready(start + Duration::from_millis(9)));
    }

    #[test]
    fn scene_preparation_retries_do_not_pay_another_presentation_interval() {
        let start = Instant::now();
        let interval = frame_interval(Some(30)).unwrap();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(start, true, interval, EmptyFrameRetry::ScenePreparation);
        let submit_vm = start + interval.unwrap();
        assert!(schedule.ready(submit_vm));
        schedule.attempted(
            submit_vm,
            false,
            interval,
            EmptyFrameRetry::ScenePreparation,
        );
        assert_eq!(schedule.delay(submit_vm), SCENE_PREPARATION_INTERVAL);
        let submit_texture = submit_vm + SCENE_PREPARATION_INTERVAL;
        assert!(schedule.ready(submit_texture));
        schedule.attempted(
            submit_texture,
            false,
            interval,
            EmptyFrameRetry::ScenePreparation,
        );
        let present = submit_texture + SCENE_PREPARATION_INTERVAL;
        assert!(schedule.ready(present));
        assert!(present - start >= interval.unwrap());
        assert!(present - submit_vm < interval.unwrap());
        schedule.attempted(present, true, interval, EmptyFrameRetry::ScenePreparation);
        assert!(!schedule.ready(present + SCENE_PREPARATION_INTERVAL));
        assert!(schedule.ready(present + interval.unwrap()));
    }

    #[test]
    fn scene_retries_preserve_the_swap_origin_when_the_fps_limit_changes() {
        let start = Instant::now();
        let interval = frame_interval(Some(30)).unwrap();
        let mut schedule = FrameSchedule::default();
        schedule.attempted(start, true, interval, EmptyFrameRetry::ScenePreparation);
        let retry = start + Duration::from_millis(40);
        schedule.attempted(retry, false, interval, EmptyFrameRetry::ScenePreparation);
        schedule.set_interval(frame_interval(Some(15)).unwrap());
        assert_eq!(
            schedule.delay(start),
            frame_interval(Some(15)).unwrap().unwrap()
        );
        schedule.set_interval(frame_interval(Some(1)).unwrap());
        assert_eq!(schedule.delay(start), Duration::from_secs(1));
        schedule.set_interval(frame_interval(Some(120)).unwrap());
        assert!(!schedule.ready(retry));
        assert!(schedule.ready(retry + SCENE_PREPARATION_INTERVAL));
        schedule.set_interval(None);
        assert_eq!(schedule.delay(retry), SCENE_PREPARATION_INTERVAL);
    }

    #[test]
    fn frequent_dispatches_and_slow_scene_decode_keep_retries_bounded() {
        let start = Instant::now();
        let interval = frame_interval(Some(30)).unwrap();
        let mut schedule = FrameSchedule::default();
        let mut attempts = Vec::new();
        let mut presentations = Vec::new();
        let mut decode_ready = start;
        // Unrelated commands/compositor events dispatch at 1kHz. Each accepted
        // Scene frame then needs 80ms of worker preparation before another swap.
        for millisecond in 0..1000 {
            let now = start + Duration::from_millis(millisecond);
            if schedule.ready(now) {
                let presented = now >= decode_ready;
                attempts.push(now);
                if presented {
                    presentations.push(now);
                    decode_ready = now + Duration::from_millis(80);
                }
                schedule.attempted(now, presented, interval, EmptyFrameRetry::ScenePreparation);
            }
        }
        assert!(presentations.len() > 10);
        assert!(attempts.len() <= 125);
        for pair in attempts.windows(2) {
            assert!(pair[1] - pair[0] >= SCENE_PREPARATION_INTERVAL);
        }
        for pair in presentations.windows(2) {
            assert!(pair[1] - pair[0] >= interval.unwrap());
        }
    }

    #[test]
    fn uncapped_scene_and_video_keep_distinct_nonzero_retry_deadlines() {
        let now = Instant::now();
        let mut scene = FrameSchedule::default();
        scene.attempted(now, false, None, EmptyFrameRetry::ScenePreparation);
        assert!(!scene.ready(now));
        assert_eq!(scene.delay(now), SCENE_PREPARATION_INTERVAL);
        let mut video = FrameSchedule::default();
        video.attempted(now, false, None, EmptyFrameRetry::Decoder);
        assert_eq!(video.delay(now), FRAME_POLL_INTERVAL);
        let interval = frame_interval(Some(30)).unwrap();
        video.attempted(now, false, interval, EmptyFrameRetry::Decoder);
        assert_eq!(video.delay(now), interval.unwrap());
        video.set_interval(frame_interval(Some(15)).unwrap());
        assert_eq!(video.delay(now), frame_interval(Some(15)).unwrap().unwrap());
    }
}
