use std::ops::Range;

use crate::vector::{Vec3, Vec4};
use crate::quaternion::Quaternion;

/// Constant for converting u64 numbers to f64s in [0,1).
/// It is the maximum value of mantissa plus one.
pub const F64_MANTISSA: f64 = (1u64 << f64::MANTISSA_DIGITS) as f64; // is 2^53


pub struct Randf32 {
    state: u64,
}
impl Randf32 {
    pub fn new() -> Self {
        Self {
            state: 555555555,
        }
    }
    pub fn next(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        ((self.state >> 11) as f64 / F64_MANTISSA) as f32
    }
    pub fn next_in(&mut self, range: Range<f32>) -> f32 {
        (range.end - range.start) * self.next() + range.start
    }
    pub fn f32_in(&mut self, bound: &(f32, f32)) -> f32 {
        bound.0 + self.next_in(-1.0..1.0) * bound.1
    }
    pub fn vec3_in(&mut self, bounds: &[(f32, f32); 3]) -> Vec3 {
        Vec3::new(
            bounds[0].0 + self.next_in(-1.0..1.0) * bounds[0].1,
            bounds[1].0 + self.next_in(-1.0..1.0) * bounds[1].1,
            bounds[2].0 + self.next_in(-1.0..1.0) * bounds[2].1,
        )
    }
    pub fn vec4_in(&mut self, bounds: &[(f32, f32); 4]) -> Vec4 {
        Vec4::new(
            bounds[0].0 + self.next_in(-1.0..1.0) * bounds[0].1,
            bounds[1].0 + self.next_in(-1.0..1.0) * bounds[1].1,
            bounds[2].0 + self.next_in(-1.0..1.0) * bounds[2].1,
            bounds[3].0 + self.next_in(-1.0..1.0) * bounds[3].1,
        )
    }
    pub fn quat_in(&mut self, bounds: &[(f32, f32); 4]) -> Quaternion {
        Quaternion::new(
            bounds[0].0 + self.next_in(-1.0..1.0) * bounds[0].1,
            bounds[1].0 + self.next_in(-1.0..1.0) * bounds[1].1,
            bounds[2].0 + self.next_in(-1.0..1.0) * bounds[2].1,
            bounds[3].0 + self.next_in(-1.0..1.0) * bounds[3].1,
        )
    }
}
impl Default for Randf32 {
    fn default() -> Self {
        Self::new()
    }
}