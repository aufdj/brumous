use std::ops::Range;
use cgmath::{Vector3, Vector4, Quaternion};

use crate::particle_system::Spread;

/// Constant for converting u64 numbers to f64s in [0,1).
/// It is the maximum value of mantissa plus one.
pub const F64_MANTISSA: f64 = (1u64 << f64::MANTISSA_DIGITS) as f64; // is 2^53


pub struct Randf32 {
    state: u64,
}
#[allow(dead_code)]
impl Randf32 {
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
    pub fn next(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        ((self.state >> 11) as f64 / F64_MANTISSA) as f32
    }
    pub fn in_range(&mut self, range: Range<f32>) -> f32 {
        (range.end - range.start) * self.next() + range.start
    }
    pub fn spread(&mut self, spread: &Spread) -> f32 {
        spread.mean + self.in_range(-1.0..1.0) * spread.variance
    }
    pub fn vec3_spread(&mut self, spread: &[Spread; 3]) -> Vector3<f32> {
        Vector3::new(
            spread[0].mean + self.in_range(-1.0..1.0) * spread[0].variance,
            spread[1].mean + self.in_range(-1.0..1.0) * spread[1].variance,
            spread[2].mean + self.in_range(-1.0..1.0) * spread[2].variance,
        )
    }
    pub fn vec4_spread(&mut self, spread: &[Spread; 4]) -> Vector4<f32> {
        Vector4::new(
            spread[0].mean + self.in_range(-1.0..1.0) * spread[0].variance,
            spread[1].mean + self.in_range(-1.0..1.0) * spread[1].variance,
            spread[2].mean + self.in_range(-1.0..1.0) * spread[2].variance,
            spread[3].mean + self.in_range(-1.0..1.0) * spread[3].variance,
        )
    }
    pub fn quat_spread(&mut self, spread: &[Spread; 4]) -> Quaternion<f32> {
        Quaternion::new(
            spread[0].mean + self.in_range(-1.0..1.0) * spread[0].variance,
            spread[1].mean + self.in_range(-1.0..1.0) * spread[1].variance,
            spread[2].mean + self.in_range(-1.0..1.0) * spread[2].variance,
            spread[3].mean + self.in_range(-1.0..1.0) * spread[3].variance,
        )
    }
}