use crate::vector::Vec3;

#[derive(Clone, Copy, Default)]
pub struct Quaternion {
    pub s: f32,
    pub v: Vec3,
}
impl Quaternion {
    pub fn new(s: f32, xi: f32, yj: f32, zk: f32) -> Self {
        Self {
            s, 
            v: Vec3::new(xi, yj, zk),
        }
    }
    pub fn zero() -> Self {
        Self {
            s: 0.0,
            v: Vec3::zero(),
        }
    }
}