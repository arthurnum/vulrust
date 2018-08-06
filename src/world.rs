use cgmath::{Matrix4, Vector4, Rad, InnerSpace};


const DEFAULT_DIRECTION: Vector4<f32> = Vector4 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
    w: 1.0
};

pub struct World {
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub model: Matrix4<f32>,
    pub direction_angle: f32
}

impl World {
    pub fn move_forwards(&mut self) {
        self._move(0.2);
    }

    pub fn move_backwards(&mut self) {
        self._move(-0.2);
    }

    pub fn rotate_clockwise(&mut self) {
        self._rotate(0.02);
    }

    pub fn rotate_counterclockwise(&mut self) {
        self._rotate(-0.02);
    }

    fn _move(&mut self, k: f32) {
        let mut direction = (Matrix4::from_angle_y(Rad(self.direction_angle)) * DEFAULT_DIRECTION).truncate().normalize() * k;
        direction.x *= -1.0;
        self.model = self.model * Matrix4::from_translation(direction);
    }

    fn _rotate(&mut self, k: f32) {
        self.direction_angle += k;
        self.model = Matrix4::from_angle_y(Rad(k)) * self.model;
    }
}
