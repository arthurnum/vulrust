#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate cgmath;
extern crate time;
extern crate rand;

use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
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

mod global;
use global::*;

mod math_utils;
mod shader_utils;
mod vertex_types;

mod rectangle_instance_builder;
use rectangle_instance_builder::RectangleInstanceBuilder;

mod rectangle_instance;
use rectangle_instance::RectangleInstance;

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
    let physical_device = {
        let mut physical_devices = PhysicalDevice::enumerate(&instance);
        // physical_devices.next().unwrap();
        physical_devices.next().unwrap()
    };
    println!("{:?}", physical_device.name());

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
    let window_builder = WindowBuilder::new().with_dimensions(SCR_WIDTH as u32, SCR_HEIGHT as u32);
    let mut events_loop = EventsLoop::new();
    let surface = window_builder.build_vk_surface(&events_loop, instance.clone()).unwrap();

    /* ##########
    SWAPCHAIN
    ########## */
    let caps = surface.capabilities(physical_device).unwrap();
    let dimensions = caps.current_extent.unwrap_or([SCR_WIDTH as u32, SCR_HEIGHT as u32]);
    let buffers_count = caps.min_image_count;
    let (format, _color_space) = caps.supported_formats[0];
    let usage = ImageUsage {
        color_attachment: true,
        .. ImageUsage::none()
    };
    let sharing_mode = SharingMode::Exclusive(present_queue.family().id());
    let present_mode = {
        let cap_present_modes = &caps.present_modes;
        if cap_present_modes.immediate { vulkano::swapchain::PresentMode::Immediate }
        else if cap_present_modes.mailbox { vulkano::swapchain::PresentMode::Mailbox }
        else { vulkano::swapchain::PresentMode::Fifo }
    };
    println!("PresentMode: {:?}", present_mode);

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
        present_mode,
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

    let mut rectangle = GfxObject::new(device.clone(), render_pass.clone());
    rectangle.create_rectangle(1.0, 1.0);

    let mut rectangle_instances: Vec<RectangleInstance> = Vec::new();
    for _i in 0..1 {
        rectangle_instances.push(RectangleInstanceBuilder::create(
            [
                0.0,
                0.0,
                -2.0
            ],
            [
                rand::random::<f32>() / 2.0,
                rand::random::<f32>() / 2.0,
                rand::random::<f32>() / 2.0
            ]
        ));
    }

    let instances_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        rectangle_instances.iter().map(|ri| {
            ri.get_instance_vertex()
        })
    ).unwrap();

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

    let mut delta: f32 = 0.0;
    let delta_uniform_pool = CpuBufferPool::new(device.clone(), BufferUsage::all());


    loop {
        previous_frame_end.cleanup_finished();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (index, acq_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let c_color = [
            1.0 * (frame_counter as f32 % 200.0 / 200.0), 1.0, 0.0
        ].into();

        delta += 1.50;
        let delta_buffer = delta_uniform_pool.next(shader_utils::vs::ty::DeltaUniform {
            delta: (delta % 630.0) / 100.0
        }).unwrap();
        let delta_descriptor_set = Arc::new(
            PersistentDescriptorSet::start(rectangle.get_pipeline(), 1)

            .add_buffer(delta_buffer)
            .unwrap()

            .build()
            .unwrap()
        );

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), present_queue.family()).unwrap()
            .begin_render_pass(framebuffers[index].clone(), false, vec![c_color, 1f32.into()])
            .unwrap();


        command_buffer_builder = command_buffer_builder.draw(
            rectangle.get_pipeline(),
            DynamicState {
                line_width: None,
                viewports: current_viewport.clone(),
                scissors: None,
            },
            (rectangle.get_vertex_buffer(), instances_buffer.clone()),
            (rectangle.get_descriptor_set_collection(), delta_descriptor_set.clone()),
            ()
        ).unwrap();


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
