use std::time::{Duration, Instant};

pub struct Delta {
    pub last_frame: Instant,
    pub frame_time: Duration,
}
impl Delta {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_time: Duration::ZERO,
        }
    }
    pub fn update(&mut self, new_frame: Instant) {
        self.frame_time = self.last_frame.elapsed();
        self.last_frame = new_frame;
    }
    pub fn frame_time(&self) -> Duration {
        self.frame_time
    }
    pub fn frame_time_f32(&self) -> f32 {
        self.frame_time.as_millis() as f32 / 1000.0
    }
}