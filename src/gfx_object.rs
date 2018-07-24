use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use math_utils;
use shader_utils;


#[derive(Debug, Clone)]
pub struct Vertex2D { pub position: [f32; 2] }
impl_vertex!(Vertex2D, position);

type SBuffer = SingleBufferDefinition<Vertex2D>;
type BPipeline = Box<PipelineLayoutAbstract + Send + Sync>;
type RPass = Arc<RenderPassAbstract + Send + Sync>;

pub struct GfxObject {
    pub device: Arc<Device>,
    pub render_pass: RPass,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex2D]>>>,
    pub pipeline: Option<Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>>>,
    pub descriptor_set_collection: Option<Vec<Arc<DescriptorSet + Send + Sync>>>
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

    pub fn create_rectangle(&mut self, lb: [f32; 2], rt: [f32; 2])
    {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            vec![
                Vertex2D { position: lb },
                Vertex2D { position: [rt[0], lb[1]] },
                Vertex2D { position: [lb[0], rt[1]] },
                Vertex2D { position: rt },
            ].into_iter()
        ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);

        let vs = shader_utils::vs::Shader::load(self.device.clone()).expect("failed to create shader module");
        let fs = shader_utils::fs::Shader::load(self.device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(self.render_pass.clone(), 0).expect("render pass failed");
        let pipeline: Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>> = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex2D>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(subpass)
            .build(self.device.clone())
            .expect("render pass failed")
        );

        let color_uniform_buffer = CpuBufferPool::new(self.device.clone(), BufferUsage::all());
        let color_uniform_subbuffer = color_uniform_buffer.next(shader_utils::fs::ty::MetaColor {
            incolor: [0.8, 0.2, 0.4, 1.0]
        }).unwrap();

        let ortho_matrix_buffer = CpuAccessibleBuffer::from_data(
            self.device.clone(),
            BufferUsage::all(),
            shader_utils::vs::ty::UniformMatrices {
                world: math_utils::ortho(400.0, 200.0).into()
            }
        ).unwrap();

        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)

            .add_buffer(ortho_matrix_buffer)
            .unwrap()

            .build()
            .unwrap()
        );

        let descriptor_set_color = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 1)

            .add_buffer(color_uniform_subbuffer)
            .unwrap()

            .build()
            .unwrap()
        );

        let descriptor_set_collection: Vec<Arc<DescriptorSet + Send + Sync>> = vec![descriptor_set, descriptor_set_color];

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

    pub fn get_vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex2D]>>
    {
        match self.vertex_buffer {
            Some(ref vertex_buffer) => { vertex_buffer.clone() }
            None => { panic!("Empty vertex buffer!") }
        }
    }

    pub fn get_descriptor_set_collection(&self) -> (Arc<DescriptorSet + Send + Sync>, Arc<DescriptorSet + Send + Sync>)
    {
        match self.descriptor_set_collection {
            Some(ref descriptor_set_collection) => {
                (
                    descriptor_set_collection[0].clone(),
                    descriptor_set_collection[1].clone()
                )
            }
            None => { panic!("Empty descriptor set collection!") }
        }
    }
}
