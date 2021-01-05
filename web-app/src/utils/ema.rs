use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use yew::services::ConsoleService;

use web_sys::{Performance, Window};

const MOVING_AVERAGE_P: f64 = 0.20;

#[derive(Clone)]
pub struct ExponentialMovingAverage {
    performance: Performance,

    download_time: Arc<AtomicUsize>,

    moving_average: Arc<AtomicUsize>,
}

impl ExponentialMovingAverage {
    pub fn new(window: &Window) -> Self {
        Self {
            performance: window.performance().expect("Can't get perf"),

            download_time: Arc::new(AtomicUsize::new(0)),
            moving_average: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn start_timer(&self) {
        let time_stamp = self.performance.now() as usize;

        self.download_time.store(time_stamp, Ordering::Relaxed);
    }

    /// Returns the newly calculated average if start_timer() was previously called
    pub fn recalculate_average(&self) -> Option<f64> {
        let old_time_stamp = self.download_time.swap(0, Ordering::Relaxed);

        if old_time_stamp == 0 {
            return None;
        }

        let new_time_stamp = self.performance.now() as usize;

        let time = (new_time_stamp - old_time_stamp) as f64;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Last Download {}ms", time));

        let mut moving_average = self.moving_average.load(Ordering::Relaxed) as f64;

        if moving_average > 0.0 {
            moving_average += (time - moving_average) * MOVING_AVERAGE_P;
        } else {
            moving_average = time;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Moving Average {}ms", moving_average));

        self.moving_average
            .store(moving_average as usize, Ordering::Relaxed);

        Some(moving_average)
    }
}
