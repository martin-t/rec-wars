//! Rudimentary time tracking, FPS counter and performance tracker.

use std::{collections::VecDeque, fmt::Debug};

// TODO do we need a trait when we have features?
pub trait Time: Debug {
    fn now(&self) -> f64;
}

/// I want to track update and render time in Rust so i can draw the FPS counter and keep stats.
/// Unfortunately, Instant::now() panics in WASM so i have to use performance.now().
/// And just like in JS, it has limited precision in some browsers like firefox.
#[cfg(feature = "raw_canvas")]
#[derive(Debug, Clone)]
pub(crate) struct RawCanvasTime(pub(crate) web_sys::Performance);

#[cfg(feature = "raw_canvas")]
impl Time for RawCanvasTime {
    fn now(&self) -> f64 {
        self.0.now()
    }
}

#[cfg(feature = "mq")]
#[derive(Debug, Clone)]
pub struct MacroquadTime;

#[cfg(feature = "mq")]
impl Time for MacroquadTime {
    fn now(&self) -> f64 {
        macroquad::time::get_time()
    }
}

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
pub(crate) struct Durations(VecDeque<f64>);

impl Durations {
    pub(crate) fn new() -> Self {
        Self(VecDeque::new())
    }

    /// Add another data point.
    ///
    /// `samples`: How many data points to keep.  
    /// `duration`: The new data point.  
    pub(crate) fn add(&mut self, samples_max: usize, duration: f64) {
        if self.0.len() >= samples_max {
            self.0.pop_front();
        }
        self.0.push_back(duration);
    }

    pub(crate) fn get_stats(&self) -> Option<(f64, f64)> {
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
