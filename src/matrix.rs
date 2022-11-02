use std::ops::Mul;

use crate::vector::{Vec3, Vec4};
use crate::quaternion::Quaternion;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Mat4x4 {
    pub c0: Vec4,
    pub c1: Vec4,
    pub c2: Vec4,
    pub c3: Vec4,
}
impl Mat4x4 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c0r0: f32, c0r1: f32, c0r2: f32, c0r3: f32,
        c1r0: f32, c1r1: f32, c1r2: f32, c1r3: f32,
        c2r0: f32, c2r1: f32, c2r2: f32, c2r3: f32,
        c3r0: f32, c3r1: f32, c3r2: f32, c3r3: f32,
    ) -> Self {
        Self::from_cols(
            Vec4::new(c0r0, c0r1, c0r2, c0r3),
            Vec4::new(c1r0, c1r1, c1r2, c1r3),
            Vec4::new(c2r0, c2r1, c2r2, c2r3),
            Vec4::new(c3r0, c3r1, c3r2, c3r3),
        )
    }
    
    pub fn from_cols(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Self { c0, c1, c2, c3 }
    }

    pub fn identity() -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn from_translation(v: Vec3) -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            v.x, v.y, v.z, 1.0
        )
    }

    pub fn from_scale(s: f32) -> Self {
        Self::from_nonuniform_scale(s, s, s)
    }

    pub fn from_nonuniform_scale(s1: f32, s2: f32, s3: f32) -> Self {
        Self::new(
            s1, 0.0, 0.0, 0.0,
            0.0, s2, 0.0, 0.0,
            0.0, 0.0, s3, 0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }
}

impl From<[[f32; 4]; 4]> for Mat4x4 {
    fn from(mat: [[f32; 4]; 4]) -> Mat4x4 {
        Self::from_cols(
            Vec4::new(mat[0][0], mat[0][1], mat[0][2], mat[0][3]),
            Vec4::new(mat[1][0], mat[1][1], mat[1][2], mat[1][3]),
            Vec4::new(mat[2][0], mat[2][1], mat[2][2], mat[2][3]),
            Vec4::new(mat[3][0], mat[3][1], mat[3][2], mat[3][3]),
        )
    }
}

impl From<Mat4x4> for [[f32; 4]; 4] {
    fn from(mat: Mat4x4) -> [[f32; 4]; 4] {
        let c0 = [mat.c0.x, mat.c0.y, mat.c0.z, mat.c0.w];
        let c1 = [mat.c1.x, mat.c1.y, mat.c1.z, mat.c1.w];
        let c2 = [mat.c2.x, mat.c2.y, mat.c2.z, mat.c2.w];
        let c3 = [mat.c3.x, mat.c3.y, mat.c3.z, mat.c3.w];
        
        [c0, c1, c2, c3]
    }
}

impl From<Quaternion> for Mat4x4 {
    /// Convert the quaternion to a 4 x 4 rotation matrix.
    fn from(quat: Quaternion) -> Mat4x4 {
        let x2 = quat.v.x + quat.v.x;
        let y2 = quat.v.y + quat.v.y;
        let z2 = quat.v.z + quat.v.z;

        let xx2 = x2 * quat.v.x;
        let xy2 = x2 * quat.v.y;
        let xz2 = x2 * quat.v.z;

        let yy2 = y2 * quat.v.y;
        let yz2 = y2 * quat.v.z;
        let zz2 = z2 * quat.v.z;

        let sy2 = y2 * quat.s;
        let sz2 = z2 * quat.s;
        let sx2 = x2 * quat.s;

        Mat4x4::new(
            1.0 - yy2 - zz2, xy2 + sz2,       xz2 - sy2,       0.0,
            xy2 - sz2,       1.0 - xx2 - zz2, yz2 + sx2,       0.0,
            xz2 + sy2,       yz2 - sx2,       1.0 - xx2 - yy2, 0.0,
            0.0,             0.0,             0.0,             1.0,
        )
    }
}

impl Mul for Mat4x4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let v0 = other.c0 * self;
        let v1 = other.c1 * self;
        let v2 = other.c2 * self;
        let v3 = other.c3 * self;

        Mat4x4::from_cols(v0, v1, v2, v3)
    }
}

#[test]
fn mat_multiply() {
    let mat1 = Mat4x4::new(
        1.0, 5.0, 9.0, 4.0,
        2.0, 6.0, 1.0, 5.0,
        3.0, 7.0, 2.0, 6.0,
        4.0, 8.0, 3.0, 7.0
    );
    let mat2 = Mat4x4::new(
        10.0, 14.0, 18.0, 12.0,
        11.0, 15.0, 19.0, 13.0,
        12.0, 16.0, 10.0, 14.0,
        13.0, 17.0, 11.0, 15.0
    );
    let res = mat1 * mat2;
    
    assert!(res == Mat4x4::new(
        140.0, 356.0, 176.0, 302.0,
        150.0, 382.0, 191.0, 324.0,
        130.0, 338.0, 186.0, 286.0,
        140.0, 364.0, 201.0, 308.0
    ));
}


#[derive(Clone, Copy, PartialEq)]
pub struct Mat3x3 {
    pub c0: Vec3,
    pub c1: Vec3,
    pub c2: Vec3,
}
impl Mat3x3 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c0r0: f32, c0r1: f32, c0r2: f32,
        c1r0: f32, c1r1: f32, c1r2: f32,
        c2r0: f32, c2r1: f32, c2r2: f32,
    ) -> Self {
        Self::from_cols(
            Vec3::new(c0r0, c0r1, c0r2),
            Vec3::new(c1r0, c1r1, c1r2),
            Vec3::new(c2r0, c2r1, c2r2),
        )
    }
    
    pub fn from_cols(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Self {
            c0,
            c1,
            c2,
        }
    }

    pub fn identity() -> Self {
        Self::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        )
    }

    pub fn from_scale(s: f32) -> Self {
        Self::from_nonuniform_scale(s, s, s)
    }

    pub fn from_nonuniform_scale(s1: f32, s2: f32, s3: f32) -> Self {
        Self::new(
            s1, 0.0, 0.0,
            0.0, s2, 0.0,
            0.0, 0.0, s3,
        )
    }
}

impl From<[[f32; 3]; 3]> for Mat3x3 {
    fn from(mat: [[f32; 3]; 3]) -> Mat3x3 {
        Self::from_cols(
            Vec3::new(mat[0][0], mat[0][1], mat[0][2]),
            Vec3::new(mat[1][0], mat[1][1], mat[1][2]),
            Vec3::new(mat[2][0], mat[2][1], mat[2][2]),
        )
    }
}

impl From<Mat3x3> for [[f32; 3]; 3] {
    fn from(mat: Mat3x3) -> [[f32; 3]; 3] {
        let c0 = [mat.c0.x, mat.c0.y, mat.c0.z];
        let c1 = [mat.c1.x, mat.c1.y, mat.c1.z];
        let c2 = [mat.c2.x, mat.c2.y, mat.c2.z];
        
        [c0, c1, c2]
    }
}

impl From<Quaternion> for Mat3x3 {
    /// Convert the quaternion to a 3 x 3 rotation matrix.
    fn from(quat: Quaternion) -> Mat3x3 {
        let x2 = quat.v.x + quat.v.x;
        let y2 = quat.v.y + quat.v.y;
        let z2 = quat.v.z + quat.v.z;

        let xx2 = x2 * quat.v.x;
        let xy2 = x2 * quat.v.y;
        let xz2 = x2 * quat.v.z;

        let yy2 = y2 * quat.v.y;
        let yz2 = y2 * quat.v.z;
        let zz2 = z2 * quat.v.z;

        let sy2 = y2 * quat.s;
        let sz2 = z2 * quat.s;
        let sx2 = x2 * quat.s;

        Mat3x3::new(
            1.0 - yy2 - zz2, xy2 + sz2,       xz2 - sy2,
            xy2 - sz2,       1.0 - xx2 - zz2, yz2 + sx2,
            xz2 + sy2,       yz2 - sx2,       1.0 - xx2 - yy2,
        )
    }
}