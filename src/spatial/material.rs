use crate::*;

use std::sync::Arc;

static mut SHARED_SAMPLER: Option<pipeline::TextureSampler> = None;

unsafe fn shared_sampler(device: &core::Device) -> &pipeline::TextureSampler {
    if let Some(sampler) = SHARED_SAMPLER.as_ref() {
        return sampler;
    } else {
        SHARED_SAMPLER = Some(pipeline::TextureSampler::new(device));
    }
    return SHARED_SAMPLER.as_ref().unwrap();
}

pub struct Material {

    pub albedo_texture: Option<Res<texture::Texture>>,
    pub normal_texture: Option<Res<texture::Texture>>,
    pub metallic_texture: Option<Res<texture::Texture>>,
    pub roughness_texture: Option<Res<texture::Texture>>,

    pub albedo_global: OpaqueColor,
    pub metallic_global: f32,
    pub roughness_global: f32,

}

impl Material {

    pub fn new(albedo_texture: Option<Res<texture::Texture>>, normal_texture: Option<Res<texture::Texture>>, metallic_texture: Option<Res<texture::Texture>>, roughness_texture: Option<Res<texture::Texture>>, albedo_global: OpaqueColor, metallic_global: f32, roughness_global: f32) -> Self {
        return Self { albedo_texture, normal_texture, metallic_texture, roughness_texture, albedo_global, metallic_global, roughness_global };
    }

    pub fn color(color: OpaqueColor, metallic: f32, roughness: f32) -> Self {
        return Self::new(None, None, None, None, color, metallic, roughness);
    }

    pub fn create_buffer(&self, graphics: &mut render::Graphics) -> MaterialBuffer {
        let mut options: i32 = 0;
        let mut albedo: Option<Arc<buffer::TextureBuffer>> = None;
        let mut normal: Option<Arc<buffer::TextureBuffer>> = None;
        let mut metallic: Option<Arc<buffer::TextureBuffer>> = None;
        let mut roughness: Option<Arc<buffer::TextureBuffer>> = None;

        if let Some(tex) = self.albedo_texture.as_ref() {
            albedo = Some(Arc::new(buffer::TextureBuffer::create(tex, &mut graphics.device)));
            options |= ShaderData::USE_ALBEDO_BIT;
        }
        if let Some(tex) = self.normal_texture.as_ref() {
            normal = Some(Arc::new(buffer::TextureBuffer::create(tex, &mut graphics.device)));
            options |= ShaderData::USE_NORMAL_BIT;
        }
        if let Some(tex) = self.metallic_texture.as_ref() {
            metallic = Some(Arc::new(buffer::TextureBuffer::create(tex, &mut graphics.device)));
            options |= ShaderData::USE_METALLIC_BIT;
        }
        if let Some(tex) = self.roughness_texture.as_ref() {
            roughness = Some(Arc::new(buffer::TextureBuffer::create(tex, &mut graphics.device)));
            options |= ShaderData::USE_ROUGHNESS_BIT;
        }

        let shader_data = ShaderData::new(options, self.albedo_global, self.metallic_global, self.roughness_global);
        let texture_buffers: MaterialTextureBuffers = MaterialTextureBuffers { albedo, normal, metallic, roughness };
        let data_buffer = buffer::Buffer::alloc_uniform(&[shader_data], &graphics.device);

        return MaterialBuffer::new(texture_buffers, data_buffer, &graphics.device).expect("Failed to create material buffer!");
    }

}

pub struct MaterialComponent {

    pub material: Material,
    pub buffer: Arc<MaterialBuffer>,

}

impl MaterialComponent {

    pub fn new(material: Material, graphics: &mut render::Graphics) -> Self {
        let buffer = Arc::new(material.create_buffer(graphics));
        return Self { material, buffer };
    }

    pub fn write_buffers(&mut self, graphics: &mut render::Graphics) {
        self.buffer = Arc::new(self.material.create_buffer(graphics));
    }

}

impl specs::Component for MaterialComponent {
    type Storage = specs::DenseVecStorage<Self>;
}

impl scene::ComponentOf<spatial::Spatial> for MaterialComponent {}

pub type ShaderOptions = i32;

#[derive(Copy, Clone)]
pub struct ShaderData {

    pub albedo_global: OpaqueColor,
    pub options: ShaderOptions,
    pub metallic_global: f32,
    pub roughness_global: f32,

}

impl ShaderData {

    pub const USE_ALBEDO_BIT: i32 =     0b00000001;
    pub const USE_NORMAL_BIT: i32 =     0b00000010;
    pub const USE_METALLIC_BIT: i32 =   0b00000100;
    pub const USE_ROUGHNESS_BIT: i32 =  0b00001000;

    pub fn new(options: ShaderOptions, albedo_global: OpaqueColor, metallic_global: f32, roughness_global: f32) -> Self {
        return Self { options, albedo_global, metallic_global, roughness_global };
    }

}

pub struct MaterialTextureBuffers {

    pub albedo: Option<Arc<buffer::TextureBuffer>>,
    pub normal: Option<Arc<buffer::TextureBuffer>>,
    pub metallic: Option<Arc<buffer::TextureBuffer>>,
    pub roughness: Option<Arc<buffer::TextureBuffer>>,

}

impl MaterialTextureBuffers {

    pub fn new(albedo: Option<Arc<buffer::TextureBuffer>>, normal: Option<Arc<buffer::TextureBuffer>>, metallic: Option<Arc<buffer::TextureBuffer>>, roughness: Option<Arc<buffer::TextureBuffer>>) -> Self {
        return Self { albedo, normal, metallic, roughness };
    }

    pub fn none() -> Self {
        return Self::new(None, None, None, None);
    }

    pub fn from_albedo(albedo: Arc<buffer::TextureBuffer>) -> Self {
        return Self { albedo: Some(albedo), normal: None, metallic: None, roughness: None };
    }

}

pub struct MaterialBuffer {

    pub texture_buffers: MaterialTextureBuffers,
    pub data_buffer: buffer::Buffer,
    pub descriptor_set: pipeline::DescriptorSet,

}


impl MaterialBuffer {

    fn create_desc_set(device: &core::Device) -> Result<pipeline::DescriptorSet, &'static str> {
        if let Some(layout) = unsafe { spatial::pipe::mesh::MATERIAL_DESCRIPTOR_LAYOUT.as_ref() } {
            let mut descriptor_pool: pipeline::DescriptorPool = pipeline::DescriptorPool::new(1, &[
                (layout.as_ref(), 1)
            ], device);
            let descriptor_set: pipeline::DescriptorSet = pipeline::DescriptorSet::new(layout.as_ref(), &mut descriptor_pool,device)?;
            return Ok(descriptor_set);
        }
        return Err("Failed to create material buffer - the mesh render pipeline has not yet been initialised.");
    }

    pub fn new(texture_buffers: MaterialTextureBuffers, data_buffer: buffer::Buffer, device: &core::Device) -> Result<Self, &'static str> {
        let descriptor_set = Self::create_desc_set(device)?;
        let this = Self {
            texture_buffers,
            data_buffer,
            descriptor_set
        };
        this.write_descriptor_input(device);
        return Ok(this);
    }

    pub fn empty(device: &core::Device) -> Result<Self, &'static str> {
        let data_buffer = buffer::Buffer::alloc_uniform(&[0i32], device);
        return Self::new(MaterialTextureBuffers::none(), data_buffer, device);
    }

    pub fn basic_albedo(albedo: Arc<buffer::TextureBuffer>, metallic: f32, roughness: f32, device: &core::Device) -> Result<Self, &'static str> {
        let shader_data = ShaderData::new(ShaderData::USE_ALBEDO_BIT, OpaqueColor::black(), metallic, roughness);
        let data_buffer = buffer::Buffer::alloc_uniform(&[shader_data], device);
        return Self::new(MaterialTextureBuffers::from_albedo(albedo), data_buffer, device);
    }

    pub fn with_color(color: OpaqueColor, metallic: f32, roughness: f32, device: &core::Device) -> Result<Self, &'static str> {
        let shader_data = ShaderData::new(0, color, metallic, roughness);
        let data_buffer = buffer::Buffer::alloc_uniform(&[shader_data], device);
        return Self::new(MaterialTextureBuffers::none(), data_buffer, device);
    }

    pub fn write_descriptor_input(&self, device: &core::Device) {
        self.descriptor_set.write_input(&self.data_buffer, 0, device);
        self.descriptor_set.write_input(unsafe { shared_sampler(device) }, 1, device);
        if let Some(tex) = self.texture_buffers.albedo.as_ref() {
            self.descriptor_set.write_input(tex.as_ref(), 2, device);
        }
        if let Some(tex) = self.texture_buffers.normal.as_ref() {
            self.descriptor_set.write_input(tex.as_ref(), 3, device);
        }
        if let Some(tex) = self.texture_buffers.metallic.as_ref() {
            self.descriptor_set.write_input(tex.as_ref(), 4, device);
        }
        if let Some(tex) = self.texture_buffers.roughness.as_ref() {
            self.descriptor_set.write_input(tex.as_ref(), 5, device);
        }
    }
}