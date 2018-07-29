#[derive(Debug, Clone)]
pub struct Vertex3D {
    pub position: [f32; 3]
}
impl_vertex!(Vertex3D, position);

#[derive(Debug, Clone)]
pub struct Vertex3DColor3D {
    pub instance_position: [f32; 3],
    pub instance_color: [f32; 3]
}
impl_vertex!(Vertex3DColor3D, instance_position, instance_color);
