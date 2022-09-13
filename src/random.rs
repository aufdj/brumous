/// Constant for converting u64 numbers to f64s in [0,1).
/// It is the maximum value of mantissa plus one.
pub const F64_MANTISSA: f64 = (1u64 << f64::MANTISSA_DIGITS) as f64; // is 2^53
pub const F32_MANTISSA: f32 = (1u64 << f32::MANTISSA_DIGITS) as f32; // is 2^53


pub struct Randf64 {
    state: u64,
}
#[allow(dead_code)]
impl Randf64 {
    pub fn new() -> Self {
        Self {
            state: 555555555,
        }
    }
    pub fn seed(seed: u64) -> Self {
        Self {
            state: seed,
        }
    }
    pub fn next(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 11) as f64 / F64_MANTISSA
    }
    pub fn in_range(&mut self, min: f64, max: f64) -> f64 {
        (max - min) * self.next() + min
    }
}


pub struct Randf32 {
    state: u32,
}
#[allow(dead_code)]
impl Randf32 {
    pub fn new() -> Self {
        Self {
            state: 555555555,
        }
    }
    pub fn seed(seed: u32) -> Self {
        Self {
            state: seed,
        }
    }
    pub fn next(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 11) as f32 / F32_MANTISSA
    }
    pub fn in_range(&mut self, min: f32, max: f32) -> f32 {
        (max - min) * self.next() + min
    }
}