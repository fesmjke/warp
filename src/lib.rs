pub mod camera;
pub mod canvas;
pub mod color;
mod entity;
pub mod float_eq;
pub mod hit;
pub mod material;
pub mod point;
pub mod preset;
pub mod ray;
pub mod utils;
pub mod vec3;
pub mod vector;

pub mod objects {
    pub mod plane;
    pub mod sphere;
    pub mod triangle;
}

pub mod matrices {
    pub use matrix::Matrix;
    pub use matrix2::Matrix2;
    pub use matrix3::Matrix3;
    pub use matrix4::Matrix4;
    mod matrix;
    mod matrix2;
    mod matrix3;
    mod matrix4;
}
