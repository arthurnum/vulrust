#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;


use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::image::ImageUsage;
use vulkano::framebuffer::Framebuffer;
use vulkano::instance::DeviceExtensions;
use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::Swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::SharingMode;
use vulkano_win::VkSurfaceBuild;
use winit::EventsLoop;
use winit::WindowBuilder;

mod init;

fn main() {
    let instance_extensions = InstanceExtensions::supported_by_core().unwrap();
    let instance = Instance::new(None, &instance_extensions, None).unwrap();

    let instance_cloned = instance.clone();
    let physical_device = PhysicalDevice::enumerate(&instance_cloned).next().unwrap();

    let (device, mut queue_iter) = {
        let queue_family = physical_device.queue_families().next().unwrap();
        let features = Features::none();
        let ext = DeviceExtensions {
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };

        match Device::new(physical_device, &features, &ext, Some((queue_family, 1.0))) {
            Ok(d) => d,
            Err(err) => panic!("Couldn't build device: {:?}", err)
        }
    };

    let present_queue = queue_iter.next().unwrap();

    // let device_cloned = device.clone();
    // let device_local_buffer: Arc<DeviceLocalBuffer<f32>> = {
    //   DeviceLocalBuffer::new(device_cloned, BufferUsage::vertex_buffer(), physical_device.queue_families()).unwrap()
    // };

    let window_builder = WindowBuilder::new().with_dimensions(400, 200);
    let mut events_loop = EventsLoop::new();
    let surface = window_builder.build_vk_surface(&events_loop, instance).unwrap();

    let caps = surface.capabilities(physical_device).unwrap();
    let dimensions = caps.current_extent.unwrap_or([640, 480]);
    let buffers_count = 2;
    let (format, _color_space) = caps.supported_formats[0];

    let usage = ImageUsage {
        color_attachment: true,
        .. ImageUsage::none()
    };

    let sharing_mode = SharingMode::Exclusive(present_queue.family().id());

    // Create the swapchain and its buffers.
    let (swapchain, buffers) = Swapchain::new(
        // Create the swapchain in this `device`'s memory.
        device.clone(),
        // The surface where the images will be presented.
        surface,
        // How many buffers to use in the swapchain.
        buffers_count,
        // The format of the images.
        format,
        // The size of each image.
        dimensions,
        // How many layers each image has.
        1,
        // What the images are going to be used for.
        usage,
        // Describes which queues will interact with the swapchain.
        sharing_mode,
        // What transformation to use with the surface.
        vulkano::swapchain::SurfaceTransform::Identity,
        // How to handle the alpha channel.
        vulkano::swapchain::CompositeAlpha::Opaque,
        // How to present images.
        vulkano::swapchain::PresentMode::Immediate,
        // Clip the parts of the buffer which aren't visible.
        true,
        // No previous swapchain.
        None
    ).unwrap();

    let render_pass = Arc::new(single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    let mut frame_counter = 1;

    let framebuffer_first = Arc::new(
        Framebuffer::start(render_pass.clone())
        .add(buffers[0].clone()).unwrap()
        .build().unwrap()
    );

    let framebuffer_second = Arc::new(
        Framebuffer::start(render_pass.clone())
        .add(buffers[1].clone()).unwrap()
        .build().unwrap()
    );

    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    loop {
        previous_frame_end.cleanup_finished();

        let (index, acq_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let current_framebuffer = {
            if index == 0 {
                framebuffer_first.clone()
            } else {
                framebuffer_second.clone()
            }
        };

        let huy = [
            1.0 * (frame_counter as f32 % 20000.0 / 20000.0), 1.0, 0.0
        ].into();

        let command_buffer = AutoCommandBufferBuilder::new(device.clone(), present_queue.family()).unwrap()
        .begin_render_pass(current_framebuffer, false, vec![huy]).unwrap()
        .end_render_pass().unwrap()
        .build().unwrap();


        let future = previous_frame_end.join(acq_future)
            .then_execute(present_queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(present_queue.clone(), swapchain.clone(), index)
            .then_signal_fence_and_flush().unwrap();

        previous_frame_end = Box::new(future) as Box<_>;

        // Handling the window events in order to close the program when the user wants to close
        // it.
        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
                _ => ()
            }
        });
        if done { return; }

        println!("Frame #{:?}", frame_counter);
        frame_counter += 1;
    }

}
