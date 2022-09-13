// use std::ops::{Neg, Add, Sub, Mul, Div};
// use std::fmt;

// use crate::random::Randf64;

// pub type Point3 = Vec3;
// pub type Color = Vec3;

// #[derive(PartialEq, Clone, Copy, Default)]
// pub struct Vec3 {
//     pub x: f64,
//     pub y: f64,
//     pub z: f64,
// }
// impl Vec3 {
//     pub fn from(x: f64, y: f64, z: f64) -> Self {
//         Self {
//             x, y, z,
//         }
//     }
//     pub fn len(&self) -> f64 {
//         self.len_sq().sqrt()
//     }
//     pub fn len_sq(&self) -> f64 {
//         self.x*self.x + self.y*self.y + self.z*self.z
//     }
//     pub fn dot(&self, vec: &Vec3) -> f64 {
//         self.x*vec.x + self.y*vec.y + self.z*vec.z
//     }
//     pub fn cross(&self, vec: &Vec3) -> Self {
//         Self::from(
//             self.y*vec.z - self.z*vec.y,
//             self.z*vec.x - self.x*vec.z,
//             self.x*vec.y - self.y*vec.x,
//         )
//     }
//     pub fn norm(&self) -> Self {
//         let len = self.len();
//         Self::from(self.x / len, self.y / len, self.z / len)
//     }
//     pub fn random_in_range(rand: &mut Randf64, min: f64, max: f64) -> Self {
//         Self {
//             x: rand.in_range(min, max),
//             y: rand.in_range(min, max),
//             z: rand.in_range(min, max),
//         }
//     }
//     pub fn random_in_unit_sphere(rand: &mut Randf64) -> Self {
//         loop {
//             let vec = Self::random_in_range(rand, -1.0, 1.0);
//             if vec.len_sq() <= 1.0 {
//                 return vec;
//             }
//         }
//     }
//     pub fn random_unit_vec(rand: &mut Randf64) -> Self {
//         Self::random_in_unit_sphere(rand).norm()
//     }
//     pub fn random_in_hemisphere(rand: &mut Randf64, normal: Vec3) -> Self {
//         let in_unit_sphere = Self::random_in_unit_sphere(rand);
//         if in_unit_sphere.dot(&normal) >= 0.0 {
//             return in_unit_sphere;
//         }
//         else {
//             return -in_unit_sphere;
//         }
//     }
//     pub fn near_zero(&self) -> bool {
//         let s = 0.00000001;
//         self.x.abs() < s && self.y.abs() < s && self.z.abs() < s
//     }
//     pub fn reflect(&self, normal: &Vec3) -> Self {
//         let b = self.dot(normal);
//         *self - (*normal * (b * 2.0))
//     }
// }
// impl Neg for Vec3 {
//     type Output = Self;

//     fn neg(self) -> Self::Output {
//         Self::from(-self.x, -self.y, -self.z)
//     }
// }
// impl Add for Vec3 {
//     type Output = Self;

//     fn add(self, other: Self) -> Self::Output {
//         Self::from(self.x + other.x, self.y + other.y, self.z + other.z)
//     }
// }
// impl Sub for Vec3 {
//     type Output = Self;

//     fn sub(self, other: Self) -> Self::Output {
//         Self::from(self.x - other.x, self.y - other.y, self.z - other.z)
//     }
// }
// impl Mul for Vec3 {
//     type Output = Self;

//     fn mul(self, other: Self) -> Self::Output {
//         Self::from(self.x - other.x, self.y - other.y, self.z - other.z)
//     }
// }
// impl Mul<f64> for Vec3 {
//     type Output = Self;

//     fn mul(self, other: f64) -> Self::Output {
//         Self::from(self.x * other, self.y * other, self.z * other)
//     }
// }
// impl Div<f64> for Vec3 {
//     type Output = Self;

//     fn div(self, other: f64) -> Self::Output {
//         Self::from(self.x / other, self.y / other, self.z / other)
//     }
// }
// impl Div for Vec3 {
//     type Output = Self;

//     fn div(self, other: Self) -> Self::Output {
//         Self::from(self.x / other.x, self.y / other.y, self.z / other.z)
//     }
// }
// impl fmt::Display for Vec3 {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "
//             \rx: {}
//             \ry: {},
//             \rz: {}\n",
//             self.x,
//             self.y,
//             self.z
//         )
//     }
// }


// #[test]
// fn cross_test() {
//     let v  = Vec3::from(4.0, 3.0, 6.0);
//     let v2 = Vec3::from(2.0, 9.0, 3.0);

//     let cross = v.cross(&v2);
//     assert!(cross == Vec3::from(-45.0, 0.0, 30.0));
// }

// #[test]
// fn dot_test() {
//     let v  = Vec3::from(4.0, 3.0, 6.0);
//     let v2 = Vec3::from(2.0, 9.0, 3.0);

//     let dot = v.dot(&v2);
//     assert!(dot == 53.0);
// }

// #[test]
// fn norm_test() {
//     let v  = Vec3::from(4.0, 3.0, 6.0);

//     assert!(v.norm().len() == 1.0);
// }




// #[derive(PartialEq, Clone, Copy, Default)]
// pub struct Vec4 {
//     pub x: f64,
//     pub y: f64,
//     pub z: f64,
//     pub w: f64,
// }
// impl Vec3 {
//     pub fn from(x: f64, y: f64, z: f64) -> Self {
//         Self {
//             x, 
//             y, 
//             z,
//             w: 1.0,
//         }
//     }
//     pub fn len(&self) -> f64 {
//         self.len_sq().sqrt()
//     }
//     pub fn len_sq(&self) -> f64 {
//         self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
//     }
//     pub fn dot(&self, vec: &Vec3) -> f64 {
//         self.x*vec.x + self.y*vec.y + self.z*vec.z + self.w*vec.w
//     }
//     pub fn cross(&self, vec: &Vec3) -> Self {
//         Self::from(
//             self.y*vec.z - self.z*vec.y,
//             self.z*vec.x - self.x*vec.z,
//             self.x*vec.y - self.y*vec.x,
//         )
//     }
//     pub fn norm(&self) -> Self {
//         let len = self.len();
//         Self::from(self.x / len, self.y / len, self.z / len)
//     }
//     pub fn random_in_range(rand: &mut Randf64, min: f64, max: f64) -> Self {
//         Self {
//             x: rand.in_range(min, max),
//             y: rand.in_range(min, max),
//             z: rand.in_range(min, max),
//         }
//     }
//     pub fn random_in_unit_sphere(rand: &mut Randf64) -> Self {
//         loop {
//             let vec = Self::random_in_range(rand, -1.0, 1.0);
//             if vec.len_sq() <= 1.0 {
//                 return vec;
//             }
//         }
//     }
//     pub fn random_unit_vec(rand: &mut Randf64) -> Self {
//         Self::random_in_unit_sphere(rand).norm()
//     }
//     pub fn random_in_hemisphere(rand: &mut Randf64, normal: Vec3) -> Self {
//         let in_unit_sphere = Self::random_in_unit_sphere(rand);
//         if in_unit_sphere.dot(&normal) >= 0.0 {
//             return in_unit_sphere;
//         }
//         else {
//             return -in_unit_sphere;
//         }
//     }
//     pub fn near_zero(&self) -> bool {
//         let s = 0.00000001;
//         self.x.abs() < s && self.y.abs() < s && self.z.abs() < s
//     }
//     pub fn reflect(&self, normal: &Vec3) -> Self {
//         let b = self.dot(normal);
//         *self - (*normal * (b * 2.0))
//     }
// }
// impl Neg for Vec3 {
//     type Output = Self;

//     fn neg(self) -> Self::Output {
//         Self::from(-self.x, -self.y, -self.z)
//     }
// }
// impl Add for Vec3 {
//     type Output = Self;

//     fn add(self, other: Self) -> Self::Output {
//         Self::from(self.x + other.x, self.y + other.y, self.z + other.z)
//     }
// }
// impl Sub for Vec3 {
//     type Output = Self;

//     fn sub(self, other: Self) -> Self::Output {
//         Self::from(self.x - other.x, self.y - other.y, self.z - other.z)
//     }
// }
// impl Mul for Vec3 {
//     type Output = Self;

//     fn mul(self, other: Self) -> Self::Output {
//         Self::from(self.x - other.x, self.y - other.y, self.z - other.z)
//     }
// }
// impl Mul<f64> for Vec3 {
//     type Output = Self;

//     fn mul(self, other: f64) -> Self::Output {
//         Self::from(self.x * other, self.y * other, self.z * other)
//     }
// }
// impl Div<f64> for Vec3 {
//     type Output = Self;

//     fn div(self, other: f64) -> Self::Output {
//         Self::from(self.x / other, self.y / other, self.z / other)
//     }
// }
// impl Div for Vec3 {
//     type Output = Self;

//     fn div(self, other: Self) -> Self::Output {
//         Self::from(self.x / other.x, self.y / other.y, self.z / other.z)
//     }
// }
// impl fmt::Display for Vec3 {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "
//             \rx: {}
//             \ry: {},
//             \rz: {}\n", 
//             self.x, 
//             self.y,
//             self.z
//         )
//     }
// }
