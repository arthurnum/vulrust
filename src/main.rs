#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate cgmath;
extern crate time;


use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Device;
use vulkano::image::ImageUsage;
use vulkano::framebuffer::Framebuffer;
use vulkano::instance::DeviceExtensions;
use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::Swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::SharingMode;
use vulkano_win::VkSurfaceBuild;
use winit::EventsLoop;
use winit::WindowBuilder;

mod math_utils;
mod shader_utils;
mod gfx_object;
use gfx_object::GfxObject;


fn main() {
    /* ##########
    INSTANCE
    ########## */
    println!("Instance.");
    let instance_extensions = InstanceExtensions::supported_by_core().unwrap();
    let instance = Instance::new(None, &instance_extensions, None).unwrap();

    /* ##########
    PHYSICAL DEVICE
    ########## */
    println!("Physical device.");
    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    /* ##########
    DEVICE
    ########## */
    println!("Device.");
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

    /* ##########
    QUEUE
    ########## */
    println!("Queue.");
    let present_queue = queue_iter.next().unwrap();

    /* ##########
    WINDOW
    ########## */
    println!("Window.");
    let window_builder = WindowBuilder::new().with_dimensions(400, 200);
    let mut events_loop = EventsLoop::new();
    let surface = window_builder.build_vk_surface(&events_loop, instance.clone()).unwrap();

    /* ##########
    SWAPCHAIN
    ########## */
    let caps = surface.capabilities(physical_device).unwrap();
    let dimensions = caps.current_extent.unwrap_or([400, 200]);
    let buffers_count = caps.min_image_count;
    let (format, _color_space) = caps.supported_formats[0];
    let usage = ImageUsage {
        color_attachment: true,
        .. ImageUsage::none()
    };
    let sharing_mode = SharingMode::Exclusive(present_queue.family().id());

    // Create the swapchain and its buffers.
    println!("Swapchain.");
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
        vulkano::swapchain::PresentMode::Mailbox,
        // Clip the parts of the buffer which aren't visible.
        true,
        // No previous swapchain.
        None
    ).unwrap();

    /* ##########
    RENDERPASS
    ########## */
    println!("Renderpass.");
    let render_pass = Arc::new(single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: vulkano::format::Format::D16Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    ).unwrap());

    /* ##########
    FRAMEBUFFERS
    ########## */
    println!("Framebuffers.");
    let depth_buffer = vulkano::image::attachment::AttachmentImage::transient(device.clone(), dimensions, vulkano::format::D16Unorm).unwrap();
    let framebuffers: Vec<Arc<Framebuffer<_,_>>> = buffers.iter().map(|buffer|
        Arc::new(
            Framebuffer::start(render_pass.clone())
            .add(buffer.clone()).unwrap()
            .add(depth_buffer.clone()).unwrap()
            .build().unwrap()
        )
    ).collect();

    let mut rectangles: Vec<GfxObject> = Vec::new();

    for j in 0..200 {
        let i = j as f32;
        let mut rectangle = GfxObject::new(device.clone(), render_pass.clone());
        rectangle.create_rectangle([i, i], [i + 80.0, i + 40.0]);
        rectangles.push(rectangle);
    }

    /* ##########
    LOOP
    ########## */
    println!("Loop.");
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;
    let mut frame_counter = 1;
    let start_time = time::SteadyTime::now();

    let current_viewport = Some(vec![Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    }]);

    loop {
        previous_frame_end.cleanup_finished();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (index, acq_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let c_color = [
            1.0 * (frame_counter as f32 % 200.0 / 200.0), 1.0, 0.0
        ].into();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), present_queue.family()).unwrap()
            .begin_render_pass(framebuffers[index].clone(), false, vec![c_color, 1f32.into()])
            .unwrap();

        for rectangle in &rectangles {
            command_buffer_builder = command_buffer_builder.draw(
                rectangle.get_pipeline(),
                DynamicState {
                    line_width: None,
                    viewports: current_viewport.clone(),
                    scissors: None,
                },
                rectangle.get_vertex_buffer(),
                rectangle.get_descriptor_set_collection(),
                ()
            ).unwrap();
        }

        let command_buffer = command_buffer_builder
            .end_render_pass()
            .unwrap()

            .build()
            .unwrap();

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
        if done { break; }

        // println!("Frame #{:?}", frame_counter);
        frame_counter += 1;
    }

    let avg_fps = frame_counter / (time::SteadyTime::now() - start_time).num_seconds();
    println!("Average FPS: {}", avg_fps);
}
