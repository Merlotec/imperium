use crate::*;

use gfx::Device as GfxDevice;
use gfx::DescriptorPool as GfxDescriptorPool;
use std::ops::Range;

pub struct PipelineLayout {

    pub layout: <Backend as gfx::Backend>::PipelineLayout,

}

impl PipelineLayout {

    pub fn create(input_layout: &[&DescriptorSetLayout], push_constant_ranges: &[(gfx::pso::ShaderStageFlags, Range<u32>)], device: &core::Device) -> PipelineLayout {
        let mut layouts: Vec<&<Backend as gfx::Backend>::DescriptorSetLayout> = Vec::with_capacity(input_layout.len());
        for il in input_layout {
            layouts.push(&il.set_layout);
        }
        let layout = device.device.create_pipeline_layout(layouts, push_constant_ranges);
        return PipelineLayout { layout };
    }

    pub fn bind_descriptor_sets(&self, input_sets: &[&DescriptorSet], encoder: &mut command::Encoder) {
        let mut sets: Vec<&<Backend as gfx::Backend>::DescriptorSet> = Vec::with_capacity(input_sets.len());
        for set in input_sets {
            sets.push(&set.desc_set);
        }
        encoder.pass.bind_graphics_descriptor_sets(&self.layout, 0, sets, &[]);
    }

}

pub struct Pipeline {

    pub graphics_pipeline: <Backend as gfx::Backend>::GraphicsPipeline,

}

impl Pipeline {

    pub fn create(pipeline_desc: gfx::pso::GraphicsPipelineDesc<Backend>, device: &core::Device) -> Result<Pipeline, &'static str> {

        let pipeline = device.device.create_graphics_pipeline(&pipeline_desc, None);
        if let Ok(graphics_pipeline) = pipeline {
            return Ok(Pipeline { graphics_pipeline });
        } else {
            return Err("Failed to create graphics pipeline.");
        }
    }

    pub fn bind(&self, encoder: &mut command::Encoder) {
        encoder.pass.bind_graphics_pipeline(&self.graphics_pipeline);
    }

}

pub struct PipelineController {

    pub pipeline: Pipeline,
    pub layout: PipelineLayout,

}

impl PipelineController {

    pub fn new(pipeline: Pipeline, layout: PipelineLayout) -> PipelineController {
        return PipelineController { pipeline, layout };
    }

    pub fn bind_descriptor_sets(&self, input_sets: &[&DescriptorSet], encoder: &mut command::Encoder) {
        self.layout.bind_descriptor_sets(input_sets, encoder);
    }

    pub fn bind(&self, encoder: &mut command::Encoder) {
        self.pipeline.bind(encoder);
    }

}


pub struct VertexInput<'a> {

    pub vertex_buffer: &'a buffer::Buffer,
    pub index_buffer: Option<&'a buffer::Buffer>,

}

pub enum ShaderStage {

    Vertex,
    Fragment,

}

pub struct DescriptorSetLayout {

    pub set_layout: <Backend as gfx::Backend>::DescriptorSetLayout,
    pub bindings: Vec<gfx::pso::DescriptorType>,

}

impl DescriptorSetLayout {

    /// There must be the same number of shader inputs specified as in the actual descriptor set in the shader.
    /// If the data is not yet initialized, this can easily be specified - all that matters is the correct layout data is supplied.
    pub fn create(inputs: &[(&ShaderInput, ShaderStage)], device: &core::Device) -> DescriptorSetLayout {
        let mut binding_data: Vec<_> = Vec::with_capacity(inputs.len());
        let mut bindings: Vec<gfx::pso::DescriptorType> = Vec::with_capacity(inputs.len());

        let mut i = 0;

        for input in inputs.iter() {
            let binding_type = input.0.get_binding_type();
            bindings.push(binding_type);
            let stage_flags = match input.1 {
                ShaderStage::Vertex => gfx::pso::ShaderStageFlags::VERTEX,
                ShaderStage::Fragment => gfx::pso::ShaderStageFlags::FRAGMENT,
            };
            binding_data.push(gfx::pso::DescriptorSetLayoutBinding {
                binding: i,
                ty: binding_type,
                count: 1,
                stage_flags,
                immutable_samplers: false,
            });
            i += 1;
        }

        // TODO: what is a descriptor set, what is the layout?
        let set_layout = device.device.create_descriptor_set_layout(
            &binding_data,
            &[],
        );
        return DescriptorSetLayout { set_layout, bindings };
    }

}

pub struct DescriptorPool {

    pub pool: <Backend as gfx::Backend>::DescriptorPool,

}

impl DescriptorPool {

    pub fn new(max_sets: usize, input_layouts: &[(&DescriptorSetLayout, usize)], device: &core::Device) -> DescriptorPool {
        let mut desc_ranges: Vec<_> = Vec::new();
        for input_layout in input_layouts {
            for binding in input_layout.0.bindings.iter() {
                desc_ranges.push(gfx::pso::DescriptorRangeDesc {
                    ty: *binding,
                    count: input_layout.1,
                });
            }
        }

        let mut pool = device.device.create_descriptor_pool(
            max_sets,
            &desc_ranges,
        );
        return DescriptorPool { pool };
    }

}

/// Represents a the set of shader inputs.
/// In Vulkan, this maps to shader 'descriptor set' objects.
/// This struct can be used to upload data to these descriptor sets.
pub struct DescriptorSet {

    pub desc_set: <Backend as gfx::Backend>::DescriptorSet,

}

impl DescriptorSet {

    /// Creates a new descriptor set object.
    /// The layout of the set needs to be provided in order to create the actual descriptor set.
    pub fn new(input_layout: &DescriptorSetLayout, descriptor_pool: &mut DescriptorPool, device: &core::Device) -> Result<DescriptorSet, &'static str> {

        if let Ok(desc_set) = descriptor_pool.pool.allocate_set(&input_layout.set_layout) {
            return Ok(DescriptorSet { desc_set });
        } else {
             return Err("Failed to create descriptor set. Perhaps the descriptor pool was not initialized properly?");
        }

    }

    /// The inputs specified should have their corresponding bindings as the second argument of the tuple. This means that not all the inputs need to be initialized.
    pub fn with_inputs(inputs: &[(&ShaderInput, u32)], input_layout: &DescriptorSetLayout, descriptor_pool: &mut DescriptorPool, device: &core::Device) -> Result<DescriptorSet, &'static str> {

        let mut descriptor_set: DescriptorSet = Self::new(input_layout, descriptor_pool, device)?;
        {
            let mut writes: Vec<_> = Vec::with_capacity(inputs.len());

            for input in inputs.iter() {
                if let Some(descriptor) = input.0.get_descriptor() {
                    writes.push(gfx::pso::DescriptorSetWrite {
                        set: &descriptor_set.desc_set,
                        binding: input.1,
                        array_offset: 0,
                        descriptors: Some(descriptor),
                    });
                }
            }

            device.device.write_descriptor_sets(writes);
        }

        return Ok(descriptor_set);
    }

    pub fn write_inputs(&mut self, inputs: &[(&ShaderInput, u32)], device: &core::Device) {
        let mut writes: Vec<_> = Vec::with_capacity(inputs.len());
        for input in inputs.iter() {
            if let Some(descriptor) = input.0.get_descriptor() {
                writes.push(gfx::pso::DescriptorSetWrite {
                    set: &self.desc_set,
                    binding: input.1,
                    array_offset: 0,
                    descriptors: Some(descriptor),
                });
            }
        }

        device.device.write_descriptor_sets(writes);
    }

    pub fn bind(&self, pipeline_layout: &<Backend as gfx::Backend>::PipelineLayout, encoder: &mut command::Encoder) {
        encoder.pass.bind_graphics_descriptor_sets(&pipeline_layout, 0, vec![&self.desc_set], &[]);
    }

    pub fn write_descriptor(&self, descriptor: Option<gfx::pso::Descriptor<Backend>>, binding: u32, device: &core::Device) {
        device.device.write_descriptor_sets(vec![
            gfx::pso::DescriptorSetWrite {
                set: &self.desc_set,
                binding,
                array_offset: 0,
                descriptors: descriptor,
            }
        ]);
    }

    pub fn write_input(&self, shader_input: &ShaderInput, binding: u32, device: &core::Device) {
        device.device.write_descriptor_sets(vec![
            gfx::pso::DescriptorSetWrite {
                set: &self.desc_set,
                binding,
                array_offset: 0,
                descriptors: shader_input.get_descriptor(),
            }
        ]);
    }

}

pub struct DescriptorSetInterface {

    pub layout: DescriptorSetLayout,
    pub set: DescriptorSet,

}

impl DescriptorSetInterface {

    pub fn new(layout: DescriptorSetLayout, set: DescriptorSet) -> DescriptorSetInterface {
        return DescriptorSetInterface { layout, set };
    }

    pub fn bind(&self, pipeline_layout: &<Backend as gfx::Backend>::PipelineLayout, encoder: &mut command::Encoder) {
        self.set.bind(pipeline_layout, encoder);
    }

    pub fn write_descriptor(&self, descriptor: Option<gfx::pso::Descriptor<Backend>>, binding: u32, device: &core::Device) {
        self.set.write_descriptor(descriptor, binding, device);
    }

    pub fn write_input(&self, shader_input: &ShaderInput, binding: u32, device: &core::Device) {
        self.set.write_input(shader_input, binding, device);
    }

}

pub struct ShaderInputDescriptor {

    pub binding_type: gfx::pso::DescriptorType,

}

pub trait ShaderInput {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>>;
    fn get_binding_type(&self) -> gfx::pso::DescriptorType;

}

impl ShaderInputDescriptor {

    pub fn image_descriptor() -> ShaderInputDescriptor {

        return ShaderInputDescriptor {
            binding_type: gfx::pso::DescriptorType::SampledImage,
        };

    }

    pub fn sampler_descriptor() -> ShaderInputDescriptor {

        return ShaderInputDescriptor {
            binding_type: gfx::pso::DescriptorType::Sampler,
        };

    }

    pub fn uniform_buffer_descriptor() -> ShaderInputDescriptor {

        return ShaderInputDescriptor {
            binding_type: gfx::pso::DescriptorType::UniformBuffer,
        }

    }

}

impl ShaderInput for ShaderInputDescriptor {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return None;
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return self.binding_type;
    }

}

pub struct TextureSampler {

    pub sampler: <Backend as gfx::Backend>::Sampler,

}

impl TextureSampler {

    pub fn new(device: &core::Device) -> TextureSampler {
        let sampler = device.device.create_sampler(gfx::image::SamplerInfo::new(gfx::image::Filter::Linear, gfx::image::WrapMode::Clamp));
        return TextureSampler { sampler };
    }

}

impl ShaderInput for TextureSampler {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return Some(gfx::pso::Descriptor::Sampler(&self.sampler));
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return gfx::pso::DescriptorType::Sampler;
    }

}
