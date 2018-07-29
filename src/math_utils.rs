use cgmath::Angle;
use cgmath::Matrix4;
use cgmath::Deg;

use global::*;


pub fn ortho(w: f32, h: f32) -> Matrix4<f32> {
    Matrix4::new(
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

pub fn perspective() -> Matrix4<f32> {
    let f = Deg(55.0).cot();
    let aspect = SCR_WIDTH / SCR_HEIGHT;
    let far = 100.0;
    let near = 0.01;

    Matrix4::new(
        f / aspect,
        0.0,
        0.0,
        0.0,

        0.0,
        -f,
        0.0,
        0.0,

        0.0,
        0.0,
        (far + near) / (near - far),
        (2.0 * far * near) / (near - far),

        0.0,
        0.0,
        -1.0,
        0.0
    )
}
