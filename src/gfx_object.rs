use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::OneVertexOneInstanceDefinition;

use global::*;
use shader_utils;
use vertex_types::{Vertex3D, Vertex3DColor3D};
use cgmath::{Point3, Vector3, Matrix4, Matrix, Rad, perspective};


type SBuffer = OneVertexOneInstanceDefinition<Vertex3D, Vertex3DColor3D>;
type BPipeline = Box<PipelineLayoutAbstract + Send + Sync>;
type RPass = Arc<RenderPassAbstract + Send + Sync>;
type ADescriptorSet = Arc<DescriptorSet + Send + Sync>;

pub struct GfxObject {
    pub device: Arc<Device>,
    pub render_pass: RPass,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex3D]>>>,
    pub pipeline: Option<Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>>>,
    pub descriptor_set_collection: Option<Vec<ADescriptorSet>>
}

impl GfxObject {
    pub fn new(device: Arc<Device>, render_pass: RPass) -> GfxObject {
        GfxObject {
            device: device,
            render_pass: render_pass,
            vertex_buffer: None,
            pipeline: None,
            descriptor_set_collection: None
        }
    }

    pub fn create_rectangle(&mut self, w: f32, h: f32)
    {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            vec![
                Vertex3D { position: [0.0, 0.0, 0.0] },
                Vertex3D { position: [0.0, h, 0.0] },
                Vertex3D { position: [w, 0.0, 0.0] },
                Vertex3D { position: [w, h, 0.0] },
            ].into_iter()
        ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);

        let vs = shader_utils::vs::Shader::load(self.device.clone()).expect("failed to create shader module");
        let fs = shader_utils::fs::Shader::load(self.device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(self.render_pass.clone(), 0).expect("render pass failed");
        let pipeline: Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>> = Arc::new(GraphicsPipeline::start()
            .vertex_input(OneVertexOneInstanceDefinition::<Vertex3D, Vertex3DColor3D>::new())
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(subpass)
            .build(self.device.clone())
            .expect("render pass failed")
        );

        let ortho_matrix_buffer = CpuAccessibleBuffer::from_data(
            self.device.clone(),
            BufferUsage::all(),
            shader_utils::vs::ty::UniformMatrices {
                world: perspective(Rad(1.9), SCR_WIDTH / SCR_HEIGHT, 0.01, 100.0).transpose().into(),
                look_at: Matrix4::look_at(Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, -10.0), Vector3::new(0.0, 1.0, 0.0)).transpose().into()
            }
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)

            .add_buffer(ortho_matrix_buffer)
            .unwrap()

            .build()
            .unwrap()
        );

        let descriptor_set_collection: Vec<ADescriptorSet> = vec![descriptor_set];

        self.pipeline = Some(pipeline);
        self.descriptor_set_collection = Some(descriptor_set_collection);
    }

    pub fn get_pipeline(&self) -> Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>>
    {
        match self.pipeline {
            Some(ref pipeline) => { pipeline.clone() }
            None => { panic!("Empty pipeline!") }
        }
    }

    pub fn get_vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex3D]>>
    {
        match self.vertex_buffer {
            Some(ref vertex_buffer) => { vertex_buffer.clone() }
            None => { panic!("Empty vertex buffer!") }
        }
    }

    pub fn get_descriptor_set_collection(&self) -> ADescriptorSet
    {
        match self.descriptor_set_collection {
            Some(ref descriptor_set_collection) => { descriptor_set_collection[0].clone() }
            None => { panic!("Empty descriptor set collection!") }
        }
    }
}
