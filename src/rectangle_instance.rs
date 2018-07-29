use vertex_types::Vertex3DColor3D;


pub struct RectangleInstance {
    pub position: Vec<f32>,
    pub color: Vec<f32>,
    pub instance_vertex: Option<Vertex3DColor3D>
}

impl RectangleInstance {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> RectangleInstance {
        RectangleInstance {
            position: position.to_vec(),
            color: color.to_vec(),
            instance_vertex: None
        }
    }

    pub fn build(&mut self) {
        let mut position: [f32; 3] = [0.0, 0.0, 0.0];
        position.copy_from_slice(&self.position[0..3]);

        let mut color: [f32; 3] = [0.0, 0.0, 0.0];
        color.copy_from_slice(&self.color[0..3]);

        let instance_vertex = Vertex3DColor3D {
                instance_position: position,
                instance_color: color
        };

        self.instance_vertex = Some(instance_vertex);
    }

    pub fn get_instance_vertex(&self) -> Vertex3DColor3D
    {
        match self.instance_vertex {
            Some(ref instance_vertex) => { instance_vertex.clone() }
            None => { panic!("Empty instance vector!") }
        }
    }
}
