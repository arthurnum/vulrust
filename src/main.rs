#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate winit;
extern crate cgmath;
extern crate time;
extern crate rand;
extern crate image;

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
use vulkano::image::immutable::ImmutableImage;
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
use winit::VirtualKeyCode;
use winit::dpi::LogicalSize;

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
use gfx_object::GfxObject3D;
use gfx_object::GfxObjectHMap;

mod world;
use world::World;

use cgmath::{Point3, Vector3, Matrix4, Matrix, Rad, perspective, One};


fn avoid_winit_wayland_hack() {
    println!("Force X11.");
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");
}

fn load_image_sample() -> image::RgbaImage {
    image::open("./fixtures/97295-mountain2-height-map-merged.png").unwrap().to_rgba()
}

fn main() {
    if cfg!(target_os = "linux") {
        avoid_winit_wayland_hack();
    }

    load_image_sample();

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
    let window_builder = WindowBuilder::new().with_dimensions(LogicalSize::new(SCR_WIDTH as f64, SCR_HEIGHT as f64));
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

    let mut terrain_plane = GfxObjectHMap::new(device.clone(), render_pass.clone());
    terrain_plane.create_plane_square(500, 0.15);

    let mut cube = GfxObject3D::new(device.clone(), render_pass.clone());
    cube.create_cube();

    let mut rectangle = GfxObject::new(device.clone(), render_pass.clone());
    rectangle.create_rectangle(1.0, 1.0);

    let mut rectangle_instances: Vec<RectangleInstance> = Vec::new();
    for _i in 0..100 {
        rectangle_instances.push(RectangleInstanceBuilder::create(
            [
                -5.0 + rand::random::<f32>() * 10.0,
                -5.0 + rand::random::<f32>() * 10.0,
                10.0 - rand::random::<f32>() * 20.0
            ],
            [
                rand::random::<f32>(),
                rand::random::<f32>(),
                rand::random::<f32>()
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

    let mut world = World {
        projection: perspective(Rad(1.4), SCR_WIDTH / SCR_HEIGHT, 0.01, 100.0).transpose(),
        view: Matrix4::look_at(Point3::new(2.0, -6.0, 7.0), Point3::new(2.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)).transpose(),
        model: Matrix4::one(),
        direction_angle: 0.0
    };

    let world_uniforms_buffer_pool = CpuBufferPool::new(device.clone(), BufferUsage::all());
    let mut world_uniforms_buffer = world_uniforms_buffer_pool.next(
        shader_utils::vs::ty::UniformMatrices {
            projection: world.projection.into(),
            view: world.view.into(),
            model: world.model.transpose().into()
        }
    ).unwrap();

    let mut world_uniforms_descriptor = Arc::new(
        PersistentDescriptorSet::start(rectangle.get_pipeline(), 0)

        .add_buffer(world_uniforms_buffer.clone())
        .unwrap()

        .build()
        .unwrap()
    );

    let world_uniforms_buffer_pool_cube = CpuBufferPool::new(device.clone(), BufferUsage::all());
    let mut world_uniforms_buffer_cube = world_uniforms_buffer_pool_cube.next(
        shader_utils::vs_cube::ty::UniformMatrices {
            projection: world.projection.into(),
            view: world.view.into(),
            model: world.model.transpose().into()
        }
    ).unwrap();


    let mut world_uniforms_descriptor_cube = Arc::new(
        PersistentDescriptorSet::start(cube.get_pipeline(), 0)

        .add_buffer(world_uniforms_buffer_cube.clone())
        .unwrap()

        .build()
        .unwrap()
    );

    let mut world_uniforms_descriptor_terrain_plane = Arc::new(
        PersistentDescriptorSet::start(terrain_plane.get_pipeline(), 0)

        .add_buffer(world_uniforms_buffer.clone())
        .unwrap()

        .build()
        .unwrap()
    );

    let (image_sample, image_sample_future) = {
        let _image_sample = load_image_sample();
        let (w, h) = _image_sample.dimensions();

        ImmutableImage::from_iter(
            _image_sample.into_raw().into_iter(),
            vulkano::image::Dimensions::Dim2d { width: w, height: h },
            format,
            present_queue.clone()
        ).unwrap()
    };

    let sampler = vulkano::sampler::Sampler::new(
            device.clone(),
            vulkano::sampler::Filter::Linear,
            vulkano::sampler::Filter::Linear,
            vulkano::sampler::MipmapMode::Nearest,
            vulkano::sampler::SamplerAddressMode::Repeat,
            vulkano::sampler::SamplerAddressMode::Repeat,
            vulkano::sampler::SamplerAddressMode::Repeat,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

    let image_sample_descriptor = Arc::new(
        PersistentDescriptorSet::start(terrain_plane.get_pipeline(), 1)

        .add_sampled_image(image_sample.clone(), sampler.clone())
        .unwrap()

        .build()
        .unwrap()
    );

    /* ##########
    LOOP
    ########## */
    println!("Loop.");
    // let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;
    let mut previous_frame_end = Box::new(image_sample_future) as Box<GpuFuture>;
    let mut frame_counter = 1;
    let start_time = time::SteadyTime::now();

    let current_viewport = Some(vec![Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    }]);

    let dynamic_state = DynamicState {
        line_width: None,
        viewports: current_viewport.clone(),
        scissors: None,
    };

    let mut delta: f32 = 0.0;
    let delta_uniform_pool = CpuBufferPool::new(device.clone(), BufferUsage::all());
    let mut world_updated = false;
    let mut pressed_keys: Vec<Option<VirtualKeyCode>> = Vec::new();

    loop {
        previous_frame_end.cleanup_finished();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let (index, acq_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let c_color = [
            1.0 * (frame_counter as f32 % 1200.0 / 1200.0),
            1.0 * (frame_counter as f32 % 120.0 / 120.0),
            1.0 * (frame_counter as f32 % 2000.0 / 2000.0)
        ].into();

        delta += 2.0;
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
            &dynamic_state,
            (rectangle.get_vertex_buffer(), instances_buffer.clone()),
            (world_uniforms_descriptor.clone(), delta_descriptor_set.clone()),
            ()
        ).unwrap();

        command_buffer_builder = command_buffer_builder.draw(
            cube.get_pipeline(),
            &dynamic_state,
            cube.get_vertex_buffer(),
            world_uniforms_descriptor_cube.clone(),
            ()
        ).unwrap();

        command_buffer_builder = command_buffer_builder.draw(
            terrain_plane.get_pipeline(),
            &dynamic_state,
            terrain_plane.get_vertex_buffer(),
            (world_uniforms_descriptor_terrain_plane.clone(), image_sample_descriptor.clone()),
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
                winit::Event::WindowEvent { event, .. } => {
                    match event {
                        winit::WindowEvent::KeyboardInput { input, .. } => {
                            println!("{:?}", input);
                            match input.state {
                                winit::ElementState::Pressed => {
                                    if !pressed_keys.contains(&input.virtual_keycode) {
                                        pressed_keys.push(input.virtual_keycode);
                                    }
                                },
                                winit::ElementState::Released => {
                                    pressed_keys.retain(|&code| code != input.virtual_keycode)
                                }
                            }
                        }
                        winit::WindowEvent::CloseRequested => done = true,
                        _ => ()
                    }
                },
                _ => ()
            }
        });
        if done { break; }

        for key in pressed_keys.iter() {
            match key {
                Some(VirtualKeyCode::Up) => {
                    world.move_forwards();
                    world_updated = true;
                },
                Some(VirtualKeyCode::Down) => {
                    world.move_backwards();
                    world_updated = true;
                },
                Some(VirtualKeyCode::Right) => {
                    world.rotate_clockwise();
                    world_updated = true;
                },
                Some(VirtualKeyCode::Left) => {
                    world.rotate_counterclockwise();
                    world_updated = true;
                },
                _ => ()
            }
        }

        if world_updated {
            world_updated = false;

            world_uniforms_buffer = world_uniforms_buffer_pool.next(
                shader_utils::vs::ty::UniformMatrices {
                    projection: world.projection.into(),
                    view: world.view.into(),
                    model: world.model.transpose().into()
                }
            ).unwrap();

            world_uniforms_descriptor = Arc::new(
                PersistentDescriptorSet::start(rectangle.get_pipeline(), 0)

                .add_buffer(world_uniforms_buffer.clone())
                .unwrap()

                .build()
                .unwrap()
            );

            world_uniforms_buffer_cube = world_uniforms_buffer_pool_cube.next(
                shader_utils::vs_cube::ty::UniformMatrices {
                    projection: world.projection.into(),
                    view: world.view.into(),
                    model: world.model.transpose().into()
                }
            ).unwrap();


            world_uniforms_descriptor_cube = Arc::new(
                PersistentDescriptorSet::start(cube.get_pipeline(), 0)

                .add_buffer(world_uniforms_buffer_cube.clone())
                .unwrap()

                .build()
                .unwrap()
            );

            world_uniforms_descriptor_terrain_plane = Arc::new(
                PersistentDescriptorSet::start(terrain_plane.get_pipeline(), 0)

                .add_buffer(world_uniforms_buffer.clone())
                .unwrap()

                .build()
                .unwrap()
            );
        }

        // println!("Frame #{:?}", frame_counter);
        frame_counter += 1;
    }

    let avg_fps = frame_counter / (time::SteadyTime::now() - start_time).num_seconds();
    println!("Average FPS: {}", avg_fps);
}
