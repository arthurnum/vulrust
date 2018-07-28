#[derive(Debug, Clone)]
pub struct Vertex2D {
    pub position: [f32; 2]
}
impl_vertex!(Vertex2D, position);

#[derive(Debug, Clone)]
pub struct Vertex2DColor3D {
    pub instance_position: [f32; 2],
    pub instance_color: [f32; 3]
}
impl_vertex!(Vertex2DColor3D, instance_position, instance_color);
