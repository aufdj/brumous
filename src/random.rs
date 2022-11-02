use std::ops::Range;

use crate::MVar;
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
    pub fn next_in(&mut self, range: Range<f32>) -> f32 {
        (range.end - range.start) * self.next() + range.start
    }
    pub fn in_variance(&mut self, mvar: &MVar) -> f32 {
        mvar.0 + self.next_in(-1.0..1.0) * mvar.1
    }
    pub fn vec3_in_variance(&mut self, mvar: &[MVar; 3]) -> Vec3 {
        Vec3::new(
            mvar[0].0 + self.next_in(-1.0..1.0) * mvar[0].1,
            mvar[1].0 + self.next_in(-1.0..1.0) * mvar[1].1,
            mvar[2].0 + self.next_in(-1.0..1.0) * mvar[2].1,
        )
    }
    pub fn vec4_in_variance(&mut self, mvar: &[MVar; 4]) -> Vec4 {
        Vec4::new(
            mvar[0].0 + self.next_in(-1.0..1.0) * mvar[0].1,
            mvar[1].0 + self.next_in(-1.0..1.0) * mvar[1].1,
            mvar[2].0 + self.next_in(-1.0..1.0) * mvar[2].1,
            mvar[3].0 + self.next_in(-1.0..1.0) * mvar[3].1,
        )
    }
    pub fn quat_in_variance(&mut self, mvar: &[MVar; 4]) -> Quaternion {
        Quaternion::new(
            mvar[0].0 + self.next_in(-1.0..1.0) * mvar[0].1,
            mvar[1].0 + self.next_in(-1.0..1.0) * mvar[1].1,
            mvar[2].0 + self.next_in(-1.0..1.0) * mvar[2].1,
            mvar[3].0 + self.next_in(-1.0..1.0) * mvar[3].1,
        )
    }
}
impl Default for Randf32 {
    fn default() -> Self {
        Self::new()
    }
}