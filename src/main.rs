#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate cgmath;


use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::image::ImageUsage;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::DeviceExtensions;
use vulkano::instance::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::Swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::sync::SharingMode;
use vulkano_win::VkSurfaceBuild;
use winit::EventsLoop;
use winit::WindowBuilder;

mod shader_utils;


#[derive(Debug, Clone)]
struct Vertex2D { position: [f32; 2] }
impl_vertex!(Vertex2D, position);

fn ortho(w: f32, h: f32) -> cgmath::Matrix4<f32> {
    cgmath::Matrix4::new (
        2.0 / w,
        0.0,
        0.0,
        -1.0,

        0.0,
        -2.0 / h,
        0.0,
        1.0,

        0.0,
        0.0,
        0.0,
        0.0,

        0.0,
        0.0,
        0.0,
        1.0
    )
}

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
    let buffers_count = 2;
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
        vulkano::swapchain::PresentMode::Immediate,
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

    println!("vertex buffer");
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        vec![
            Vertex2D { position: [0.0, 0.0] },
            Vertex2D { position: [200.0, 0.0] },
            Vertex2D { position: [0.0, 200.0] },
        ].into_iter()
    ).unwrap();

    println!("color uniform buffer");
    let color_uniform_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        shader_utils::fs::ty::MetaColor {
            incolor: [0.8, 0.2, 0.4, 1.0]
        }
    ).unwrap();

    println!("ortho matrix buffer");
    let ortho_matrix_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        shader_utils::vs::ty::UniformMatrices {
            world: ortho(400.0, 200.0).into()
        }
    ).unwrap();

    let vs = shader_utils::vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = shader_utils::fs::Shader::load(device.clone()).expect("failed to create shader module");

    /* ##########
    PIPELINE
    ########## */
    println!("Pipeline.");
    let subpass = Subpass::from(render_pass.clone(), 0).expect("render pass failed");
    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex2D>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_strip()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .depth_stencil_simple_depth()
        .render_pass(subpass)
        .build(device.clone())
        .expect("render pass failed")
    );

    println!("descriptor_set");
    let descriptor_set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)

        .add_buffer(ortho_matrix_buffer)
        .unwrap()

        .add_buffer(color_uniform_buffer)
        .unwrap()

        .build()
        .unwrap()
    );

    let mut frame_counter = 1;
    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    /* ##########
    LOOP
    ########## */
    println!("Loop.");
    loop {
        previous_frame_end.cleanup_finished();

        let (index, acq_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let c_color = [
            1.0 * (frame_counter as f32 % 20000.0 / 20000.0), 1.0, 0.0
        ].into();

        let command_buffer = AutoCommandBufferBuilder::new(device.clone(), present_queue.family()).unwrap()
        .begin_render_pass(framebuffers[index].clone(), false, vec![c_color, 1f32.into()]).unwrap()
        .draw(
            pipeline.clone(),
            DynamicState {
                line_width: None,
                // TODO: Find a way to do this without having to dynamically allocate a Vec every frame.
                viewports: Some(vec![Viewport {
                  origin: [0.0, 0.0],
                  dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                  depth_range: 0.0 .. 1.0,
                }]),
                scissors: None,
            },
            vertex_buffer.clone(),
            descriptor_set.clone(),
            ()
        ).unwrap()
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

        // println!("Frame #{:?}", frame_counter);
        frame_counter += 1;
    }

}
