use std::time::{Duration, Instant};

pub struct Delta {
    pub last_frame: Instant,
}
impl Delta {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
        }
    }
    pub fn from(&mut self, new_frame: Instant) -> Duration {
        let delta = new_frame.duration_since(self.last_frame);
        self.last_frame = new_frame;
        delta
    }
}