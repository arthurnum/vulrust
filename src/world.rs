use cgmath::Matrix4;


pub struct World {
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub model: Matrix4<f32>
}
