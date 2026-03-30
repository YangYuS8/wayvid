//! Frame timing and adaptive frame skip logic
//!
//! This module implements intelligent frame skipping to handle system overload gracefully.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Maximum number of frame samples to keep for load calculation
const FRAME_HISTORY_SIZE: usize = 60;

/// Load threshold above which we start skipping frames (80% of budget)
const OVERLOAD_THRESHOLD: f64 = 0.80;

/// Load threshold below which we stop skipping frames (60% of budget)
const RECOVERY_THRESHOLD: f64 = 0.60;

/// Minimum consecutive frames before changing skip state
const HYSTERESIS_FRAMES: usize = 3;

/// Frame timing tracker with adaptive skip logic
#[derive(Debug)]
pub struct FrameTiming {
    /// Recent frame durations for load calculation
    frame_durations: VecDeque<Duration>,

    /// Last frame start time
    last_frame_start: Instant,

    /// Current frame budget based on target FPS
    target_frame_duration: Duration,

    /// Number of frames skipped
    frames_skipped: u64,

    /// Number of frames rendered
    frames_rendered: u64,

    /// Whether we're currently in skip mode
    in_skip_mode: bool,

    /// Consecutive frames in current load state (for hysteresis)
    consecutive_state_frames: usize,

    /// Last load percentage reported
    last_load_pct: f64,
}

impl FrameTiming {
    /// Create a new frame timing tracker
    ///
    /// # Arguments
    /// * `target_fps` - Target frame rate (0 = use default 60 FPS)
    pub fn new(target_fps: u32) -> Self {
        let fps = if target_fps > 0 { target_fps } else { 60 };
        let target_frame_duration = Duration::from_secs_f64(1.0 / fps as f64);

        Self {
            frame_durations: VecDeque::with_capacity(FRAME_HISTORY_SIZE),
            last_frame_start: Instant::now(),
            target_frame_duration,
            frames_skipped: 0,
            frames_rendered: 0,
            in_skip_mode: false,
            consecutive_state_frames: 0,
            last_load_pct: 0.0,
        }
    }

    /// Record the start of a new frame
    #[inline]
    pub fn begin_frame(&mut self) {
        self.last_frame_start = Instant::now();
    }

    /// Record the completion of a frame and update statistics
    #[inline]
    pub fn end_frame(&mut self) {
        let duration = self.last_frame_start.elapsed();

        // Add to history, maintaining size limit
        if self.frame_durations.len() >= FRAME_HISTORY_SIZE {
            self.frame_durations.pop_front();
        }
        self.frame_durations.push_back(duration);

        self.frames_rendered += 1;
    }

    /// Record that a frame was skipped
    #[inline]
    pub fn record_skip(&mut self) {
        self.frames_skipped += 1;
    }

    /// Check if the next frame should be skipped based on current load
    pub fn should_skip_frame(&mut self) -> bool {
        if self.frame_durations.len() < 10 {
            return false;
        }

        let load_pct = self.get_load_percentage();
        self.last_load_pct = load_pct;

        let is_overloaded = load_pct > OVERLOAD_THRESHOLD;
        let is_recovered = load_pct < RECOVERY_THRESHOLD;

        if self.in_skip_mode {
            if is_recovered {
                self.consecutive_state_frames += 1;
            } else {
                self.consecutive_state_frames = 0;
            }

            if self.consecutive_state_frames >= HYSTERESIS_FRAMES {
                self.in_skip_mode = false;
                self.consecutive_state_frames = 0;
                debug!(
                    "🟢 Frame skip: Exiting skip mode (load: {:.1}%)",
                    load_pct * 100.0
                );
            }
        } else {
            if is_overloaded {
                self.consecutive_state_frames += 1;
            } else {
                self.consecutive_state_frames = 0;
            }

            if self.consecutive_state_frames >= HYSTERESIS_FRAMES {
                self.in_skip_mode = true;
                self.consecutive_state_frames = 0;
                warn!(
                    "🔴 Frame skip: Entering skip mode (load: {:.1}%)",
                    load_pct * 100.0
                );
            }
        }

        self.in_skip_mode
    }

    /// Calculate current load as percentage of frame budget
    #[inline]
    pub fn get_load_percentage(&self) -> f64 {
        if self.frame_durations.is_empty() {
            return 0.0;
        }

        let total: Duration = self.frame_durations.iter().sum();
        let len = self.frame_durations.len() as u32;
        let avg_duration = total / len;

        avg_duration.as_secs_f64() / self.target_frame_duration.as_secs_f64()
    }

    /// Get statistics for monitoring
    pub fn get_stats(&self) -> FrameStats {
        FrameStats {
            frames_rendered: self.frames_rendered,
            frames_skipped: self.frames_skipped,
            total_frames: self.frames_rendered + self.frames_skipped,
            skip_percentage: if self.frames_rendered + self.frames_skipped > 0 {
                (self.frames_skipped as f64 / (self.frames_rendered + self.frames_skipped) as f64)
                    * 100.0
            } else {
                0.0
            },
            current_load_pct: self.last_load_pct * 100.0,
            in_skip_mode: self.in_skip_mode,
            avg_frame_duration_ms: if !self.frame_durations.is_empty() {
                let total: Duration = self.frame_durations.iter().sum();
                (total / self.frame_durations.len() as u32).as_secs_f64() * 1000.0
            } else {
                0.0
            },
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.frames_skipped = 0;
        self.frames_rendered = 0;
        self.in_skip_mode = false;
        self.consecutive_state_frames = 0;
    }
}

/// Frame timing statistics
#[derive(Debug, Clone)]
pub struct FrameStats {
    pub frames_rendered: u64,
    pub frames_skipped: u64,
    pub total_frames: u64,
    pub skip_percentage: f64,
    pub current_load_pct: f64,
    pub in_skip_mode: bool,
    pub avg_frame_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_frame_timing_basic() {
        let mut timing = FrameTiming::new(60);

        for _ in 0..20 {
            timing.begin_frame();
            sleep(Duration::from_millis(10));
            timing.end_frame();

            assert!(!timing.should_skip_frame());
        }

        let stats = timing.get_stats();
        assert_eq!(stats.frames_rendered, 20);
        assert_eq!(stats.frames_skipped, 0);
    }
}
