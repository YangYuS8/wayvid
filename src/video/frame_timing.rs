//! Frame timing and adaptive frame skip logic
//!
//! This module implements intelligent frame skipping to handle system overload gracefully.
//! It monitors frame durations, detects overload conditions, and adapts the frame rate
//! dynamically to maintain smooth playback without stuttering.

#![allow(dead_code)]

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
    ///
    /// Uses hysteresis to avoid rapid mode switching:
    /// - Enter skip mode when load > 80% for 3+ consecutive frames
    /// - Exit skip mode when load < 60% for 3+ consecutive frames
    pub fn should_skip_frame(&mut self) -> bool {
        // Need enough history to make a decision
        if self.frame_durations.len() < 10 {
            return false;
        }

        let load_pct = self.get_load_percentage();
        self.last_load_pct = load_pct;

        // Determine if we're in high or low load state
        let is_overloaded = load_pct > OVERLOAD_THRESHOLD;
        let is_recovered = load_pct < RECOVERY_THRESHOLD;

        // Update consecutive frame counter based on current state
        if self.in_skip_mode {
            // In skip mode, check for recovery
            if is_recovered {
                self.consecutive_state_frames += 1;
            } else {
                self.consecutive_state_frames = 0;
            }

            // Exit skip mode after consecutive recovery frames
            if self.consecutive_state_frames >= HYSTERESIS_FRAMES {
                self.in_skip_mode = false;
                self.consecutive_state_frames = 0;
                debug!(
                    "🟢 Frame skip: Exiting skip mode (load: {:.1}%)",
                    load_pct * 100.0
                );
            }
        } else {
            // Not in skip mode, check for overload
            if is_overloaded {
                self.consecutive_state_frames += 1;
            } else {
                self.consecutive_state_frames = 0;
            }

            // Enter skip mode after consecutive overload frames
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
    ///
    /// Returns a value between 0.0 (no load) and >1.0 (overloaded)
    #[inline]
    pub fn get_load_percentage(&self) -> f64 {
        if self.frame_durations.is_empty() {
            return 0.0;
        }

        // Calculate average frame duration from recent history
        let total: Duration = self.frame_durations.iter().sum();
        let len = self.frame_durations.len() as u32;
        let avg_duration = total / len;

        // Convert to load percentage (avoid ms conversion)
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

    /// Reset statistics (useful for testing)
    #[allow(dead_code)]
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
    /// Total frames successfully rendered
    pub frames_rendered: u64,

    /// Total frames skipped due to overload
    pub frames_skipped: u64,

    /// Total frames (rendered + skipped)
    pub total_frames: u64,

    /// Percentage of frames skipped
    pub skip_percentage: f64,

    /// Current load as percentage (0-100+)
    pub current_load_pct: f64,

    /// Whether currently in skip mode
    pub in_skip_mode: bool,

    /// Average frame duration in milliseconds
    pub avg_frame_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_frame_timing_basic() {
        let mut timing = FrameTiming::new(60);

        // Simulate normal frames
        for _ in 0..20 {
            timing.begin_frame();
            sleep(Duration::from_millis(10)); // Under budget
            timing.end_frame();

            assert!(
                !timing.should_skip_frame(),
                "Should not skip under normal load"
            );
        }

        let stats = timing.get_stats();
        assert_eq!(stats.frames_rendered, 20);
        assert_eq!(stats.frames_skipped, 0);
    }

    #[test]
    fn test_frame_timing_overload() {
        let mut timing = FrameTiming::new(60);

        // Simulate overload
        for i in 0..20 {
            timing.begin_frame();
            sleep(Duration::from_millis(20)); // Over budget (16.67ms)
            timing.end_frame();

            let should_skip = timing.should_skip_frame();

            // Should enter skip mode after a few frames
            if i >= 12 {
                // After hysteresis
                assert!(should_skip, "Should skip after sustained overload");
            }
        }

        let stats = timing.get_stats();
        assert!(stats.in_skip_mode, "Should be in skip mode");
        assert!(stats.current_load_pct > 80.0, "Load should be high");
    }

    #[test]
    fn test_frame_timing_recovery() {
        let mut timing = FrameTiming::new(60);

        // Cause overload
        for _ in 0..15 {
            timing.begin_frame();
            sleep(Duration::from_millis(20));
            timing.end_frame();
            timing.should_skip_frame();
        }

        assert!(timing.in_skip_mode, "Should be in skip mode");

        // Recover - need enough frames to clear the history buffer (60 samples)
        for _ in 0..60 {
            timing.begin_frame();
            sleep(Duration::from_millis(8)); // Well under budget
            timing.end_frame();
            timing.should_skip_frame();
        }

        assert!(!timing.in_skip_mode, "Should exit skip mode after recovery");
    }

    #[test]
    fn test_load_percentage() {
        let mut timing = FrameTiming::new(60);

        // Simulate 50% load (8ms per frame, target is 16.67ms)
        for _ in 0..20 {
            timing.begin_frame();
            sleep(Duration::from_millis(8));
            timing.end_frame();
        }

        let load = timing.get_load_percentage();
        assert!(
            load > 0.45 && load < 0.55,
            "Load should be ~50%, got {}",
            load
        );
    }
}
