use std::time::{Duration, Instant};

pub struct Timer {
    duration: Duration,
    start_time: Instant,
    started: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
            start_time: Instant::now(),
            started: false,
        }
    }
}

impl Timer {
    pub fn reset(&mut self, duration: Duration) {
        self.duration = duration;
        self.start_time = Instant::now();
        self.started = true;
    }

    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.started = true;
    }

    pub fn pause(&mut self) {
        if self.started {
            let elapsed = self.start_time.elapsed();
            self.duration = self.duration.saturating_sub(elapsed);
        }
        self.started = false;
    }

    pub fn remaining(&self) -> Duration {
        if self.started {
            let elapsed = self.start_time.elapsed();
            self.duration.saturating_sub(elapsed)
        } else {
            self.duration
        }
    }
}
