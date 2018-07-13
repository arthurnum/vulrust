
use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::instance::PhysicalDevice;


pub fn init(instance: &'static Arc<Instance>) -> PhysicalDevice<'static> {

    PhysicalDevice::from_index(instance, 0).unwrap()
}
