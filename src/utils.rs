use std::time::{Duration, Instant};

pub type Map<K, V> = ahash::AHashMap<K, V>;

pub struct Timer {
    duration: Duration,
    start_time: Instant,
    started: bool,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            start_time: Instant::now(),
            started: true,
        }
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
