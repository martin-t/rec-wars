//! Rudimentary FPS counter and performance tracker.

use std::{collections::VecDeque, fmt::Debug};

/// Saves frame times over some period of time to measure FPS.
#[derive(Debug, Clone, Default)]
pub struct Fps(VecDeque<f64>);

/// FPS Counter
///
/// There are multiple ways to count FPS.
/// Methods like using 1 / average_ms_per_frame end up with a lot of 59.9 vs 60.1 jitter.
/// Counting number of frames during the last second seems to give a stable 60.
impl Fps {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn tick(&mut self, period: f64, real_time: f64) {
        self.0.push_back(real_time);
        while !self.0.is_empty() && self.0.front().unwrap() + period < real_time {
            self.0.pop_front();
        }
    }

    pub fn get_fps(&self) -> f64 {
        if self.0.is_empty() {
            0.0
        } else {
            let diff_time = self.0.back().unwrap() - self.0.front().unwrap();
            let diff_frames = self.0.len() - 1;
            diff_frames as f64 / diff_time
        }
    }
}

/// Track durations of some event over time, report max and average.
#[derive(Debug, Clone, Default)]
pub struct Durations(VecDeque<f64>);

impl Durations {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    /// Add another data point.
    ///
    /// `samples`: How many data points to keep.  
    /// `duration`: The new data point.  
    pub fn add(&mut self, samples_max: usize, duration: f64) {
        if self.0.len() >= samples_max {
            self.0.pop_front();
        }
        self.0.push_back(duration);
    }

    pub fn get_stats(&self) -> Option<(f64, f64)> {
        if self.0.is_empty() {
            return None;
        }

        let mut sum = 0.0;
        let mut max = 0.0;
        for &dur in &self.0 {
            sum += dur;
            if dur > max {
                max = dur;
            }
        }
        let avg = sum / self.0.len() as f64;

        Some((avg, max))
    }
}
