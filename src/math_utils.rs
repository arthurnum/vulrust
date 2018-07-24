use cgmath::Matrix4;

pub fn ortho(w: f32, h: f32) -> Matrix4<f32> {
    Matrix4::new (
        2.0 / w,
        0.0,
        0.0,
        -1.0,

        0.0,
        -2.0 / h,
        0.0,
        1.0,

        0.0,
        0.0,
        0.0,
        0.0,

        0.0,
        0.0,
        0.0,
        1.0
    )
}
