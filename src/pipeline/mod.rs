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

pub struct DescriptorBinding {

    pub ty: gfx::pso::DescriptorType,
    pub binding: u32,

}

impl DescriptorBinding {

    pub fn new(ty: gfx::pso::DescriptorType, binding: u32) -> DescriptorBinding {
        return DescriptorBinding { ty, binding };
    }

}

pub struct DescriptorSetLayout {

    pub set_layout: <Backend as gfx::Backend>::DescriptorSetLayout,
    pub bindings: Vec<DescriptorBinding>,

}

impl DescriptorSetLayout {

    pub fn create(inputs: &[(&ShaderInput, ShaderStage, u32)], device: &core::Device) -> DescriptorSetLayout {
        let mut binding_data: Vec<_> = Vec::with_capacity(inputs.len());
        let mut bindings: Vec<DescriptorBinding> = Vec::with_capacity(inputs.len());

        for input in inputs.iter() {
            let binding_type = input.0.get_binding_type();
            bindings.push(DescriptorBinding::new(binding_type, input.2));
            let stage_flags = match input.1 {
                ShaderStage::Vertex => gfx::pso::ShaderStageFlags::VERTEX,
                ShaderStage::Fragment => gfx::pso::ShaderStageFlags::FRAGMENT,
            };
            binding_data.push(gfx::pso::DescriptorSetLayoutBinding {
                binding: input.2,
                ty: binding_type,
                count: 1,
                stage_flags,
                immutable_samplers: false,
            });
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
                    ty: binding.ty,
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
    pub bindings: Vec<u32>,

}

impl DescriptorSet {

    /// Creates a new descriptor set object.
    /// The layout of the set needs to be provided in order to create the actual descriptor set.
    pub fn new(input_layout: &DescriptorSetLayout, descriptor_pool: &mut DescriptorPool, device: &core::Device) -> DescriptorSet {

        let mut bindings: Vec<u32> = Vec::with_capacity(input_layout.bindings.len());

        for binding in input_layout.bindings.iter() {
            bindings.push(binding.binding);
        }

        let desc_set = descriptor_pool.pool.allocate_set(&input_layout.set_layout).log_expect("Failed to create descriptor set. Perhaps the descriptor pool was not initialized properly?");

        return DescriptorSet { desc_set, bindings };

    }

    /// The index specified as the second component of the tuple should be relative to the DescriptorSetLayout.
    /// For example, if a descriptor was added to the DescriptorSetLayout which was at binding 5 of the shader, it would be referenced as index 0.
    /// If another descriptor was added to the DescriptorSetLayout which was at binding 3 of the shader, it would be referenced as index 1 as it is the second object added.
    /// This makes it easier when the shader layout is not known, but it means that this index must be less than the number of elements in the DescriptorSetLayout object.
    pub fn create(inputs: &[(&ShaderInput, usize)], input_layout: &DescriptorSetLayout, descriptor_pool: &mut DescriptorPool, device: &core::Device) -> DescriptorSet {

        let mut bindings: Vec<u32> = Vec::with_capacity(input_layout.bindings.len());

        for binding in input_layout.bindings.iter() {
            bindings.push(binding.binding);
        }
        // TODO: explain

        let desc_set = descriptor_pool.pool.allocate_set(&input_layout.set_layout).log_expect("Failed to create descriptor set. Perhaps the descriptor pool was not initialized properly?");

        {
            let mut writes: Vec<_> = Vec::with_capacity(inputs.len());

            for input in inputs.iter() {
                if input.0.get_descriptor().is_some() {
                    let binding = input_layout.bindings[input.1].binding;
                    writes.push(gfx::pso::DescriptorSetWrite {
                        set: &desc_set,
                        binding,
                        array_offset: 0,
                        descriptors: input.0.get_descriptor(),
                    });
                }
            }

            device.device.write_descriptor_sets(writes);
        }

        return DescriptorSet { desc_set, bindings };
    }

    pub fn bind(&self, pipeline_layout: &<Backend as gfx::Backend>::PipelineLayout, encoder: &mut command::Encoder) {
        encoder.pass.bind_graphics_descriptor_sets(&pipeline_layout, 0, vec![&self.desc_set], &[]);
    }

    pub fn write_descriptor(&self, descriptor: Option<gfx::pso::Descriptor<Backend>>, index: usize, device: &core::Device) {
        device.device.write_descriptor_sets(vec![
            gfx::pso::DescriptorSetWrite {
                set: &self.desc_set,
                binding: self.bindings[index],
                array_offset: 0,
                descriptors: descriptor,
            }
        ]);
    }

    pub fn write_input(&self, shader_input: &ShaderInput, index: usize, device: &core::Device) {
        device.device.write_descriptor_sets(vec![
            gfx::pso::DescriptorSetWrite {
                set: &self.desc_set,
                binding: self.bindings[index],
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

    pub fn write_descriptor(&self, descriptor: Option<gfx::pso::Descriptor<Backend>>, index: usize, device: &core::Device) {
        self.set.write_descriptor(descriptor, index, device);
    }

    pub fn write_input(&self, shader_input: &ShaderInput, index: usize, device: &core::Device) {
        self.set.write_input(shader_input, index, device);
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

impl ShaderInput for ShaderInputDescriptor{

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return None;
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return self.binding_type;
    }

}

pub struct Uniform {

    pub buffer: buffer::Buffer,

}

impl Uniform {

    pub fn create<T: Copy>(data: &[T], device: &core::Device) -> Uniform {
        let buffer = buffer::Buffer::create_uniform(data, device);
        return Uniform { buffer };
    }

    pub fn upload_data<T: Copy>(&self, data: &[T], device: &core::Device) {
        self.buffer.fill_buffer(data, device);
    }

}

impl ShaderInput for Uniform {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return Some(gfx::pso::Descriptor::Buffer(&self.buffer.buf, None..None));
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return gfx::pso::DescriptorType::UniformBuffer;
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
