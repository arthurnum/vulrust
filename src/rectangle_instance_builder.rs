use rectangle_instance::RectangleInstance;

pub struct RectangleInstanceBuilder;

impl RectangleInstanceBuilder {
    pub fn create(position: [f32; 3], color: [f32; 3]) -> RectangleInstance
    {
        let mut rectangle_instance = RectangleInstance::new(
            position,
            color
        );
        rectangle_instance.build();
        rectangle_instance
    }
}
