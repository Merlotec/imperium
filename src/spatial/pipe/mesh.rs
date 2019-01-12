use crate::*;

use spatial::*;
use spatial::light::LightsList;
use spatial::material::MaterialsList;

use gfx::Device as GfxDevice;

const MAX_DESCRIPTORS: usize = 1000;

const MAX_BONES: usize = 100;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoneList {

    pub count: i32,
    pub bones: [Matrix4f; MAX_BONES],

}

impl BoneList {

    pub fn new() -> BoneList {
        return BoneList { count: 0, bones: [Matrix4f::identity(); MAX_BONES] };
    }

}

/// The structure responsible for rendering mesh objects.
/// This is invoked by the scene when a mesh should be rendered.
pub struct MeshRenderPipeline {

    pub pipeline: pipeline::PipelineController,
    pub descriptor_pool: pipeline::DescriptorPool,
    pub intrinsic_descriptor_interface: pipeline::DescriptorSetInterface,
    pub texture_input_desc: pipeline::DescriptorSetLayout,
    pub bone_uniform: buffer::Buffer,

    pub is_bound: bool,
}

impl MeshRenderPipeline {

    pub fn create(device: &mut core::Device, render_pass: &render::RenderPass) -> MeshRenderPipeline {
        let mut lights_uniform = buffer::Buffer::alloc_uniform(&[LightsList::new()], device);
        let mut bone_uniform = buffer::Buffer::alloc_uniform(&[BoneList::new()], device);
        let instrinsic_set_layout = pipeline::DescriptorSetLayout::create(&[
            (&bone_uniform, pipeline::ShaderStage::Vertex),
            (&lights_uniform, pipeline::ShaderStage::Fragment),
        ], device);
        let texture_input_desc: pipeline::DescriptorSetLayout = pipeline::DescriptorSetLayout::create(&[
            (&pipeline::ShaderInputDescriptor::image_descriptor(), pipeline::ShaderStage::Fragment),
            (&pipeline::ShaderInputDescriptor::sampler_descriptor(), pipeline::ShaderStage::Fragment),
            (&pipeline::ShaderInputDescriptor::uniform_buffer_descriptor(), pipeline::ShaderStage::Fragment),
        ], device);
        log!(debug, 4, "Attempting to create descriptor sets.");

        let mut descriptor_pool: pipeline::DescriptorPool = pipeline::DescriptorPool::new(MAX_DESCRIPTORS, &[
            (&instrinsic_set_layout, 1),
            (&texture_input_desc, MAX_DESCRIPTORS - 1),
        ], device);
        let intrinsic_descriptor_set: pipeline::DescriptorSet = pipeline::DescriptorSet::with_inputs(&[
            (&bone_uniform, 0),
            (&lights_uniform, 1),
        ], &instrinsic_set_layout, &mut descriptor_pool, device
        ).log_expect("Failed to create intrinsic descriptor set for mesh render pipeline.");
        let intrinsic_descriptor_interface = pipeline::DescriptorSetInterface::new(instrinsic_set_layout, intrinsic_descriptor_set);

        log!(debug, 4, "Successfully created and allocated internal descriptor sets.");

        //shader_input.write_input(&render::TextureSampler::new(device), 3, device);

        let pipeline_layout = pipeline::PipelineLayout::create(&[&intrinsic_descriptor_interface.layout, &texture_input_desc], &[(gfx::pso::ShaderStageFlags::VERTEX, 0..(Self::num_push_constants() as u32))], device);

        let vertex_shader_module = device.load_shader_raw(include_bytes!("../../../shaders/bin/std_mesh_v.spv")).expect("Fatal Error: Failed to create model vertex shader.");
        let fragment_shader_module = device.load_shader_raw(include_bytes!("../../../shaders/bin/std_mesh_f.spv")).expect("Fatal Error: Failed to create model fragment shader.");
        let pipeline_object: pipeline::Pipeline = {
            let vs_entry = gfx::pso::EntryPoint::<backend::Backend> {
                entry: "main",
                module: &vertex_shader_module,
                specialization: Default::default(),
            };

            let fs_entry = gfx::pso::EntryPoint::<backend::Backend> {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Default::default(),
            };

            let shader_entries = gfx::pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let subpass = gfx::pass::Subpass {
                index: 0,
                main_pass: &render_pass.raw_render_pass,
            };

            let rasterizer: gfx::pso::Rasterizer = gfx::pso::Rasterizer {
                polygon_mode: gfx::pso::PolygonMode::Fill,
                cull_face: gfx::pso::Face::BACK,
                front_face: gfx::pso::FrontFace::CounterClockwise,
                depth_clamping: false,
                depth_bias: None,
                conservative: false,
            };

            let mut pipeline_desc = gfx::pso::GraphicsPipelineDesc::new(
                shader_entries,
                gfx::Primitive::TriangleList,
                rasterizer,
                &pipeline_layout.layout,
                subpass,
            );

            pipeline_desc
                .blender
                .targets
                .push(gfx::pso::ColorBlendDesc(gfx::pso::ColorMask::ALL, gfx::pso::BlendState::ALPHA));

            pipeline_desc.vertex_buffers.push(gfx::pso::VertexBufferDesc {
                binding: 0,
                stride: std::mem::size_of::<model::ModelVertex>() as u32,
                rate: 0,
            });

            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 0,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::Rgb32Float,
                    offset: 0,
                },
            });

            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 1,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::Rgb32Float,
                    offset: 12,
                },
            });
            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 2,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::Rg32Float,
                    offset: 24,
                },
            });
            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 3,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::Rgba32Int,
                    offset: 32,
                },
            });
            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 4,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::Rgba32Float,
                    offset: 48,
                },
            });
            pipeline_desc.attributes.push(gfx::pso::AttributeDesc {
                location: 5,
                binding: 0,
                element: gfx::pso::Element {
                    format: gfx::format::Format::R32Int,
                    offset: 64,
                },
            });

            pipeline_desc.depth_stencil = gfx::pso::DepthStencilDesc {
                depth: gfx::pso::DepthTest::On {
                    fun: gfx::pso::Comparison::Less,
                    write: true,
                },
                depth_bounds: true,
                stencil: gfx::pso::StencilTest::default(),
            };
            log!(debug, 3, "Attempting to create mesh render pipeline.");
            pipeline::Pipeline::create(pipeline_desc, device).log_expect("Failed to create pipeline.")
        };

        let pipeline = pipeline::PipelineController::new(pipeline_object, pipeline_layout);
        log!(debug, 3, "Successfully created mesh render pipeline.");
        return MeshRenderPipeline { pipeline, descriptor_pool, intrinsic_descriptor_interface, texture_input_desc, bone_uniform, is_bound: false };
    }

    /// Renders the vertex input data with a texture.
    /// The texture in this case part of the ShaderInputSet object.
    /// Each texture rendering object should construct on of these using the layout specified in the 'texture_input_desc' field.
    /// This layout is ()
    pub fn render(&mut self, vertex_input: &pipeline::VertexInput, texture_set: &pipeline::DescriptorSet, transform: render::RenderTransform, graphics: &mut render::Graphics, encoder: &mut command::Encoder) {
        self.pipeline.bind(encoder);
        self.pipeline.bind_descriptor_sets(&[&self.intrinsic_descriptor_interface.set, texture_set], encoder);
        encoder.pass.bind_vertex_buffers(0, vec![(&vertex_input.vertex_buffer.buf, 0)]);
        if let Some(index_buffer) = vertex_input.index_buffer {
            encoder.pass.push_graphics_constants(&self.pipeline.layout.layout, gfx::pso::ShaderStageFlags::VERTEX, 0, unsafe { std::slice::from_raw_parts(&transform as *const render::RenderTransform as *const u32, Self::num_push_constants()) });
            encoder.pass.bind_index_buffer(gfx::buffer::IndexBufferView { buffer: &index_buffer.buf, offset: 0, index_type: gfx::IndexType::U32 });
            encoder.pass.draw_indexed(0..index_buffer.count as u32, 0, 0..vertex_input.vertex_buffer.count as u32);
        } else {
            encoder.pass.push_graphics_constants(&self.pipeline.layout.layout, gfx::pso::ShaderStageFlags::VERTEX, 0, unsafe { std::slice::from_raw_parts(&transform as *const render::RenderTransform as *const u32, Self::num_push_constants()) });
            encoder.pass.draw(0..vertex_input.vertex_buffer.count as u32, 0..1);
        }


    }

    pub fn render_batch(&mut self, vertex_input: &pipeline::VertexInput, texture_set: &pipeline::DescriptorSet, transforms: &[render::RenderTransform], graphics: &mut render::Graphics, encoder: &mut command::Encoder) {
        if !self.is_bound {
            self.pipeline.bind(encoder);
            self.is_bound = true;
        }
        self.pipeline.bind_descriptor_sets(&[&self.intrinsic_descriptor_interface.set, texture_set], encoder);
        encoder.pass.bind_vertex_buffers(0, vec![(&vertex_input.vertex_buffer.buf, 0)]);

        if let Some(index_buffer) = vertex_input.index_buffer {
            encoder.pass.bind_index_buffer(gfx::buffer::IndexBufferView { buffer: &index_buffer.buf, offset: 0, index_type: gfx::IndexType::U32 });
            for transform in transforms {
                encoder.pass.push_graphics_constants(&self.pipeline.layout.layout, gfx::pso::ShaderStageFlags::VERTEX, 0, unsafe { std::slice::from_raw_parts(transform as *const render::RenderTransform as *const u32, Self::num_push_constants()) });
                encoder.pass.draw_indexed(0..index_buffer.count as u32, 0, 0..vertex_input.vertex_buffer.count as u32);
            }
        } else {
            for transform in transforms {
                encoder.pass.push_graphics_constants(&self.pipeline.layout.layout, gfx::pso::ShaderStageFlags::VERTEX, 0, unsafe { std::slice::from_raw_parts(transform as *const render::RenderTransform as *const u32, Self::num_push_constants()) });
                encoder.pass.draw(0..vertex_input.vertex_buffer.count as u32, 0..1);
            }
        }


    }

    /// Called when the renderer needs to be reset (i.e. when another renderer's context is set up).
    pub fn reset(&mut self) {
        self.is_bound = false;
    }

    const fn num_push_constants() -> usize {
        return std::mem::size_of::<render::RenderTransform>() / std::mem::size_of::<u32>();
    }

}