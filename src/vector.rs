use std::ops::{Neg, Add, AddAssign, Sub, Mul, Div, Range};
use std::iter::Sum;
use std::fmt;

use crate::random::Randf32;
use crate::matrix::Mat4x4;

#[derive(PartialEq, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn len_sq(&self) -> f32 {
        self.x*self.x + self.y*self.y + self.z*self.z
    }

    pub fn dot(&self, vec: Vec3) -> f32 {
        self.x*vec.x + self.y*vec.y + self.z*vec.z
    }

    pub fn cross(&self, vec: Vec3) -> Self {
        Self::new(
            self.y*vec.z - self.z*vec.y,
            self.z*vec.x - self.x*vec.z,
            self.x*vec.y - self.y*vec.x,
        )
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();
        Self::new(self.x / len, self.y / len, self.z / len)
    }

    pub fn random_in_range(rand: &mut Randf32, range: Range<f32>) -> Self {
        Self::new(
            rand.next_in(range.clone()),
            rand.next_in(range.clone()),
            rand.next_in(range),
        )  
    }

    pub fn random_in_unit_sphere(rand: &mut Randf32) -> Self {
        loop {
            let vec = Self::random_in_range(rand, -1.0..1.0);
            if vec.len_sq() <= 1.0 {
                return vec;
            }
        }
    }

    pub fn random_unit_vec(rand: &mut Randf32) -> Self {
        Self::random_in_unit_sphere(rand).normalized()
    }

    pub fn random_in_hemisphere(rand: &mut Randf32, normal: Vec3) -> Self {
        let in_unit_sphere = Self::random_in_unit_sphere(rand);
        if in_unit_sphere.dot(normal) >= 0.0 {
            in_unit_sphere
        }
        else {
            -in_unit_sphere
        }
    }

    pub fn near_zero(&self) -> bool {
        let s = 0.00000001;
        self.x.abs() < s && self.y.abs() < s && self.z.abs() < s
    }

    pub fn reflect(&self, normal: Vec3) -> Self {
        let b = self.dot(normal);
        *self - (normal * (b * 2.0))
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        };
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self::new(self.x * other, self.y * other, self.z * other)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self::new(self.x / other, self.y / other, self.z / other)
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(arr: [f32; 3]) -> Vec3 {
        Vec3::new(arr[0], arr[1], arr[2])
    }
}

impl<'a> Sum<&'a Vec3> for Vec3 {
    fn sum<I>(iter: I) -> Self where I: Iterator<Item = &'a Self> {
        iter.fold(Vec3::zero(), |acc, vec| acc + *vec)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "
            \rx: {}
            \ry: {},
            \rz: {}\n",
            self.x,
            self.y,
            self.z
        )
    }
}




#[test]
fn cross_test() {
    let v  = Vec3::new(4.0, 3.0, 6.0);
    let v2 = Vec3::new(2.0, 9.0, 3.0);

    let cross = v.cross(v2);
    assert!(cross == Vec3::new(-45.0, 0.0, 30.0));
}

#[test]
fn dot_test() {
    let v  = Vec3::new(4.0, 3.0, 6.0);
    let v2 = Vec3::new(2.0, 9.0, 3.0);

    let dot = v.dot(v2);
    assert!(dot == 53.0);
}


#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x, 
            y, 
            z,
            w,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn unit_y() -> Self {
        Self::new(0.0, 1.0, 0.0, 0.0)
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn len_sq(&self) -> f32 {
        self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
    }

    pub fn dot(&self, vec: Vec4) -> f32 {
        self.x*vec.x + self.y*vec.y + self.z*vec.z + self.w*vec.w
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();
        Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
    }

    pub fn random_in_range(rand: &mut Randf32, range: Range<f32>) -> Self {
        Self::new(
            rand.next_in(range.clone()),
            rand.next_in(range.clone()),
            rand.next_in(range),
            0.0,
        )   
    }

    pub fn random_in_unit_sphere(rand: &mut Randf32) -> Self {
        loop {
            let vec = Self::random_in_range(rand, -1.0..1.0);
            if vec.len_sq() <= 1.0 {
                return vec;
            }
        }
    }

    pub fn random_unit_vec(rand: &mut Randf32) -> Self {
        Self::random_in_unit_sphere(rand).normalized()
    }

    pub fn random_in_hemisphere(rand: &mut Randf32, normal: Vec4) -> Self {
        let in_unit_sphere = Self::random_in_unit_sphere(rand);
        if in_unit_sphere.dot(normal) >= 0.0 {
            in_unit_sphere
        }
        else {
            -in_unit_sphere
        }
    }

    pub fn near_zero(&self) -> bool {
        let s = 0.00000001;
        self.x.abs() < s && self.y.abs() < s && self.z.abs() < s && self.w.abs() < s
    }

    pub fn reflect(&self, normal: Vec4) -> Self {
        let b = self.dot(normal);
        *self - (normal * (b * 2.0))
    }
}

impl Neg for Vec4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl Add for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w
        };
    }
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z, self.w - other.w)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self::new(self.x * other, self.y * other, self.z * other, self.w * other)
    }
}

impl Mul<Mat4x4> for Vec4 {
    type Output = Self;

    fn mul(self, other: Mat4x4) -> Self::Output {
        let c0 = other.c0 * self.x;
        let c1 = other.c1 * self.y;
        let c2 = other.c2 * self.z;
        let c3 = other.c3 * self.w;

        let x = c0.x + c1.x + c2.x + c3.x;
        let y = c0.y + c1.y + c2.y + c3.y;
        let z = c0.z + c1.z + c2.z + c3.z;
        let w = c0.w + c1.w + c2.w + c3.w;

        Vec4::new(x, y, z, w)
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self::new(self.x / other, self.y / other, self.z / other, self.w / other)
    }
}

impl From<Vec4> for [f32; 4] {
    fn from(vec: Vec4) -> [f32; 4] {
        [vec.x, vec.y, vec.z, vec.w]
    }
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "
            \rx: {}
            \ry: {},
            \rz: {}
            \rw: {}\n", 
            self.x, 
            self.y,
            self.z,
            self.w
        )
    }
}

#[test]
fn multiply_vec_by_mat() {
    let vec = Vec4::new(10.0, 11.0, 12.0, 13.0);
    let mat = Mat4x4::new(
        1.0, 5.0, 9.0, 4.0,
        2.0, 6.0, 1.0, 5.0,
        3.0, 7.0, 2.0, 6.0,
        4.0, 8.0, 3.0, 7.0
    );
    let res = vec * mat;
    assert!(res == Vec4::new(120.0, 304.0, 164.0, 258.0));
}
