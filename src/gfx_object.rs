use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::OneVertexOneInstanceDefinition;
use vulkano::pipeline::vertex::SingleBufferDefinition;

use std::fs::File;
use std::io::Read;
use std::mem::transmute;

use shader_utils;
use vertex_types::{Vertex3D, Vertex3DColor3D, Vertex3DNormal3D, Vertex3DUV};


type SBuffer = OneVertexOneInstanceDefinition<Vertex3D, Vertex3DColor3D>;
type DOBuffer = SingleBufferDefinition<Vertex3DNormal3D>;
type UVBuffer = SingleBufferDefinition<Vertex3DUV>;
type BPipeline = Box<PipelineLayoutAbstract + Send + Sync>;
type RPass = Arc<RenderPassAbstract + Send + Sync>;

pub struct GfxObject {
    pub device: Arc<Device>,
    pub render_pass: RPass,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex3D]>>>,
    pub pipeline: Option<Arc<GraphicsPipeline<SBuffer, BPipeline, RPass>>>
}

impl GfxObject {
    pub fn new(device: Arc<Device>, render_pass: RPass) -> GfxObject {
        GfxObject {
            device: device,
            render_pass: render_pass,
            vertex_buffer: None,
            pipeline: None
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

        self.pipeline = Some(pipeline);
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
}

pub struct GfxObject3D {
    pub device: Arc<Device>,
    pub render_pass: RPass,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex3DNormal3D]>>>,
    pub pipeline: Option<Arc<GraphicsPipeline<DOBuffer, BPipeline, RPass>>>
}

impl GfxObject3D {
    pub fn new(device: Arc<Device>, render_pass: RPass) -> GfxObject3D {
        GfxObject3D {
            device: device,
            render_pass: render_pass,
            vertex_buffer: None,
            pipeline: None
        }
    }

    pub fn create_cube(&mut self)
    {
        let _vertices: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0]
        ];

        let _normals: Vec<[f32; 3]> = vec![
            [ 0.0,  0.0,  1.0],
            [ 0.0,  0.0, -1.0],
            [ 0.0,  1.0,  0.0],
            [ 0.0, -1.0,  0.0],
            [ 1.0,  0.0,  0.0],
            [-1.0,  0.0,  0.0]
        ];

        let _faces: Vec<(u32, u32)> = vec![
            (1, 2), (7, 2), (5, 2),
            (1, 2), (3, 2), (7, 2),
            (1, 6), (4, 6), (3, 6),
            (1, 6), (2, 6), (4, 6),
            (3, 3), (8, 3), (7, 3),
            (3, 3), (4, 3), (8, 3),
            (5, 5), (7, 5), (8, 5),
            (5, 5), (8, 5), (6, 5),
            (1, 4), (5, 4), (6, 4),
            (1, 4), (6, 4), (2, 4),
            (2, 1), (6, 1), (8, 1),
            (2, 1), (8, 1), (4, 1)
        ];

        // let _data: Vec<Vertex3DNormal3D> = _faces.iter().map(|&_face| {
        //     let vi: u32 = _face.0 - 1;
        //     let ni: u32 = _face.1 - 1;
        //     Vertex3DNormal3D {
        //         position: _vertices[vi as usize].clone(),
        //         normal: _normals[ni as usize].clone()
        //     }
        // }).collect();

        let mut file = File::open("cube.data").unwrap();

        let mut data: Vec<f32> = Vec::new();
        let mut double_buffer: [u8; 4] = [0; 4];

        let mut read = true;

        while read {
            match file.read(&mut double_buffer) {
                Ok(size) => {
                    if size < 1 { read = false }
                }
                Err(err) => read = false
            }

            if read {
                unsafe {
                    data.push(transmute::<[u8; 4], f32>(double_buffer));
                }
            }
        }

        let mut _data: Vec<Vertex3DNormal3D> = Vec::new();

        while !data.is_empty() {
            let v: Vec<_> = data.drain(..3).collect();
            let n: Vec<_> = data.drain(..3).collect();
            let mut vd = [0f32; 3];
            let mut nd = [0f32; 3];
            vd.copy_from_slice(v.as_slice());
            nd.copy_from_slice(n.as_slice());
            _data.push(
                Vertex3DNormal3D {
                    position: vd,
                    normal: nd
                }
            );
        }

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            _data.into_iter()
        ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);

        let vs = shader_utils::vs_cube::Shader::load(self.device.clone()).expect("failed to create shader module");
        let fs = shader_utils::fs_cube::Shader::load(self.device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(self.render_pass.clone(), 0).expect("render pass failed");
        let pipeline: Arc<GraphicsPipeline<DOBuffer, BPipeline, RPass>> = Arc::new(GraphicsPipeline::start()
            .vertex_input(SingleBufferDefinition::<Vertex3DNormal3D>::new())
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(subpass)
            .build(self.device.clone())
            .expect("render pass failed")
        );

        self.pipeline = Some(pipeline);
    }

    pub fn get_pipeline(&self) -> Arc<GraphicsPipeline<DOBuffer, BPipeline, RPass>>
    {
        match self.pipeline {
            Some(ref pipeline) => { pipeline.clone() }
            None => { panic!("Empty pipeline!") }
        }
    }

    pub fn get_vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex3DNormal3D]>>
    {
        match self.vertex_buffer {
            Some(ref vertex_buffer) => { vertex_buffer.clone() }
            None => { panic!("Empty vertex buffer!") }
        }
    }
}

pub struct GfxObjectHMap {
    pub device: Arc<Device>,
    pub render_pass: RPass,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex3DUV]>>>,
    pub pipeline: Option<Arc<GraphicsPipeline<UVBuffer, BPipeline, RPass>>>
}

impl GfxObjectHMap {
    pub fn new(device: Arc<Device>, render_pass: RPass) -> GfxObjectHMap {
        GfxObjectHMap {
            device: device,
            render_pass: render_pass,
            vertex_buffer: None,
            pipeline: None
        }
    }

    pub fn create_plane_square(&mut self, dim: u32, s: f32)
    {
        let mut _data: Vec<Vertex3DUV> = Vec::new();

        let uv_s = 1.0 / (dim as f32);
        (0 .. dim + 1).for_each(|i| {
            (0 .. dim + 1).for_each(|j| {
                let x = s * i as f32;
                let z = -s * j as f32;

                let left_bottom_vertex  = [    x, 0.0,     z];
                let left_top_vertex     = [    x, 0.0, z - s];
                let right_bottom_vertex = [x + s, 0.0,     z];
                let right_top_vertex    = [x + s, 0.0, z - s];

                let u = (i as f32) / (dim as f32);
                let v = (j as f32) / (dim as f32);

                let left_bottom_uv =  [       u,        v];
                let left_top_uv    =  [       u, v + uv_s];
                let right_bottom_uv = [u + uv_s,        v];
                let right_top_uv =    [u + uv_s, v + uv_s];

                _data.push(Vertex3DUV {
                    position: left_bottom_vertex,
                    uv: left_bottom_uv
                });
                _data.push(Vertex3DUV {
                    position: left_top_vertex,
                    uv: left_top_uv
                });
                _data.push(Vertex3DUV {
                    position: right_bottom_vertex,
                    uv: right_bottom_uv
                });

                _data.push(Vertex3DUV {
                    position: left_top_vertex,
                    uv: left_top_uv
                });
                _data.push(Vertex3DUV {
                    position: right_bottom_vertex,
                    uv: right_bottom_uv
                });
                _data.push(Vertex3DUV {
                    position: right_top_vertex,
                    uv: right_top_uv
                });
            })
        });

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            _data.into_iter()
        ).unwrap();

        self.vertex_buffer = Some(vertex_buffer);

        let vs = shader_utils::vs_plane_hmap::Shader::load(self.device.clone()).expect("failed to create shader module");
        let fs = shader_utils::fs_plane_hmap::Shader::load(self.device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(self.render_pass.clone(), 0).expect("render pass failed");
        let pipeline: Arc<GraphicsPipeline<UVBuffer, BPipeline, RPass>> = Arc::new(GraphicsPipeline::start()
            .vertex_input(SingleBufferDefinition::<Vertex3DUV>::new())
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(subpass)
            .build(self.device.clone())
            .expect("render pass failed")
        );

        self.pipeline = Some(pipeline);
    }

    pub fn get_pipeline(&self) -> Arc<GraphicsPipeline<UVBuffer, BPipeline, RPass>>
    {
        match self.pipeline {
            Some(ref pipeline) => { pipeline.clone() }
            None => { panic!("Empty pipeline!") }
        }
    }

    pub fn get_vertex_buffer(&self) -> Arc<CpuAccessibleBuffer<[Vertex3DUV]>>
    {
        match self.vertex_buffer {
            Some(ref vertex_buffer) => { vertex_buffer.clone() }
            None => { panic!("Empty vertex buffer!") }
        }
    }
}
