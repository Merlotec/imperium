use crate::*;

use std::marker::PhantomData;
use std::sync::Arc;
use std::ops::Deref;

use gfx::Device as GfxDevice;
use gfx::PhysicalDevice;

pub trait BufferInterface : Send + Sync {
    fn raw_buffer(&self) -> &Buffer;
}

/// A buffer that can be shared between pipelines, resources or components.
/// For example, a `lights` buffer may be a `SharedBuffer` as it needs to be shared between all the pipelines.
pub type SharedBuffer = Arc<Buffer>;

/// This structure represents a GPU buffer which contains memory of a specific type.
/// This holds both the buffer object and the buffer memory.
pub struct Buffer {

    pub buf: <Backend as gfx::Backend>::Buffer,
    pub memory: <Backend as gfx::Backend>::Memory,
    pub count: usize,

    device_token: core::DeviceToken,

}

impl Buffer {

    pub fn alloc_vertex<T: std::marker::Copy>(slice: &[T], device: &core::Device) -> Self {
        return Self::alloc(slice, gfx::buffer::Usage::VERTEX, gfx::memory::Properties::CPU_VISIBLE, device);
    }

    pub fn alloc_uniform<T: std::marker::Copy>(slice: &[T], device: &core::Device) -> Self {
        return Self::alloc(slice, gfx::buffer::Usage::UNIFORM, gfx::memory::Properties::CPU_VISIBLE, device);
    }

    pub fn alloc_uniform_empty<T: std::marker::Copy>(count: usize, device: &core::Device) -> Self {
        return Self::alloc_empty::<T>(count, gfx::buffer::Usage::UNIFORM, gfx::memory::Properties::CPU_VISIBLE, device);
    }

    pub fn alloc<T: std::marker::Copy>(slice: &[T], usage: gfx::buffer::Usage, properties: gfx::memory::Properties, device: &core::Device) -> Self {
        let mut buffer = Self::alloc_empty::<T>(slice.len(), usage, properties, device);
        buffer.fill_buffer(slice, device);
        return buffer;
    }

    pub fn alloc_empty<T: std::marker::Copy>(count: usize, usage: gfx::buffer::Usage, properties: gfx::memory::Properties, device: &core::Device) -> Self {
        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let stride = std::mem::size_of::<T>() as u64;
        let buffer_len = count as u64 * stride;
        unsafe {
            let mut buffer = device.gpu
                .create_buffer(buffer_len, usage)
                .unwrap();

            let (upload_type, req) = device.upload_type_for(&buffer, properties);

            let buffer_memory = device.gpu.allocate_memory(upload_type, req.size).unwrap();

            device.gpu
                .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
                .unwrap();

            return Self { buf: buffer, memory: buffer_memory, count, device_token: device.create_token() };
        }

    }

    /// It is the responsibility of the programmer to ensure the data is of the right size and format.
    pub fn fill_buffer<T: Copy>(&mut self, data: &[T], device: &core::Device) {
        let stride = std::mem::size_of::<T>() as u64;
        let buffer_len = data.len() as u64 * stride;
        unsafe {
            let mut dest = device.gpu.acquire_mapping_writer::<T>(&self.memory, 0..buffer_len).unwrap();
            dest.copy_from_slice(data);
            device.gpu.release_mapping_writer(dest);
        }
        self.count = data.len();
    }

}

impl std::ops::Drop for Buffer {

    fn drop(&mut self) {
        unsafe {
            use std::mem;
            self.device_token.gpu.destroy_buffer(mem::transmute_copy(&self.buf));
            self.device_token.gpu.free_memory(mem::transmute_copy(&self.memory));
        }
    }

}

impl BufferInterface for Buffer {
    fn raw_buffer(&self) -> &Buffer {
        return self;
    }
}

impl pipeline::ShaderInput for Buffer {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return Some(gfx::pso::Descriptor::Buffer(&self.buf, None..None));
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return gfx::pso::DescriptorType::UniformBuffer;
    }

}

pub struct TextureBuffer {

    pub image: <Backend as gfx::Backend>::Image,
    pub memory: <Backend as gfx::Backend>::Memory,
    pub image_view: <Backend as gfx::Backend>::ImageView,

}

impl TextureBuffer {

    pub fn empty(graphics: &mut render::Graphics) -> Self {
        return Self::create(&texture::Texture::new(), &mut graphics.device);
    }

    pub fn new(size: Vector2u, format: gfx::format::Format, usage: gfx::image::Usage, aspects: gfx::format::Aspects, device: &core::Device) -> Self {

        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let kind = gfx::image::Kind::D2(size.x, size.y, 1, 1);

        unsafe {
            let mut image = device.gpu
                .create_image(
                    kind,
                    1,
                    format,
                    gfx::image::Tiling::Optimal,
                    usage,
                    gfx::image::ViewCapabilities::empty(),
                ).expect("Failed to create unbound image");

            let image_req = device.gpu.get_image_requirements(&image);

            let device_type = memory_types
                .iter()
                .enumerate()
                .position(|(id, memory_type)| {
                    image_req.type_mask & (1 << id) != 0
                        && memory_type.properties.contains(gfx::memory::Properties::DEVICE_LOCAL)
                }).unwrap()
                .into();

            let memory = device.gpu
                .allocate_memory(device_type, image_req.size)
                .expect("Failed to allocate image");

            device.gpu
                .bind_image_memory(&memory, 0, &mut image)
                .expect("Failed to bind image");

            let image_view = device.gpu
                .create_image_view(
                    &image,
                    gfx::image::ViewKind::D2,
                    format,
                    gfx::format::Swizzle::NO,
                    gfx::image::SubresourceRange {
                        aspects,
                        levels: 0..1,
                        layers: 0..1,
                    },
                ).expect("Failed to create image view");

            return Self { image, memory, image_view };
        }

    }

    pub fn create_depth(size: Vector2u, depth_format: gfx::format::Format, device: &core::Device) -> Self {
        Self::new(size, depth_format, gfx::image::Usage::DEPTH_STENCIL_ATTACHMENT, gfx::format::Aspects::DEPTH | gfx::format::Aspects::STENCIL, device)
    }

    pub fn create(texture: &texture::Texture, device: &mut core::Device) -> TextureBuffer {

        let texture_fence = device.gpu.create_fence(false);
        let (width, height) = (texture.dimensions.x, texture.dimensions.y);

        let mut texture_buffer = Self::new(
            Vector2u::new(width, height),
            gfx::format::Format::Rgba8Srgb,
            gfx::image::Usage::TRANSFER_DST | gfx::image::Usage::SAMPLED,
            gfx::format::Aspects::COLOR,
            &device
        );

        texture_buffer.upload_texture(texture, device);

        return texture_buffer;
    }

    pub fn upload_texture(&mut self, texture: &texture::Texture, device: &mut core::Device) {
        let texture_fence = device.gpu.create_fence(false).unwrap();

        let (width, height) = (texture.dimensions.x, texture.dimensions.y);
        let row_alignment_mask = device.adapter.physical_device.limits().min_buffer_copy_pitch_alignment as u32 - 1;
        let image_stride = 4usize;
        let row_pitch =
            (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
        let upload_size = u64::from(height * row_pitch);

        let upload_buffer = Buffer::alloc_empty::<u8>(
            upload_size as usize,
            gfx::buffer::Usage::TRANSFER_SRC,
            gfx::memory::Properties::CPU_VISIBLE,
            &device
        );
        {
            let mut data = unsafe { device.gpu
                .acquire_mapping_writer::<u8>(&upload_buffer.memory, 0..upload_size)
                .unwrap() };

            for y in 0..height as usize {
                let row = &(*texture.data)[y * (width as usize) * image_stride
                    ..(y + 1) * (width as usize) * image_stride];
                let dest_base = y * row_pitch as usize;
                data[dest_base..dest_base + row.len()].copy_from_slice(row);
            }

            unsafe { device.gpu.release_mapping_writer(data) };
        }

        let cmd_buffer = unsafe {
            let mut cmd_pool =
                device.gpu.create_command_pool_typed(
                    &device.queue_group,
                    gfx::pool::CommandPoolCreateFlags::empty())
                    .expect("Failed to create stating command pool for buffer::TextureBuffer");

            let mut cmd_buffer = cmd_pool.acquire_command_buffer::<gfx::command::OneShot>();

            let image_barrier = gfx::memory::Barrier::Image {
                states: (gfx::image::Access::empty(), gfx::image::Layout::Undefined)
                    ..(gfx::image::Access::TRANSFER_WRITE, gfx::image::Layout::TransferDstOptimal),
                target: &self.image,
                families: None,
                range: gfx::image::SubresourceRange {
                    aspects: gfx::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                },
            };

            cmd_buffer.pipeline_barrier(
                gfx::pso::PipelineStage::TOP_OF_PIPE..gfx::pso::PipelineStage::TRANSFER,
                gfx::memory::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &upload_buffer.buf,
                &self.image,
                gfx::image::Layout::TransferDstOptimal,
                &[gfx::command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / (image_stride as u32),
                    buffer_height: height as u32,
                    image_layers: gfx::image::SubresourceLayers {
                        aspects: gfx::format::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: gfx::image::Offset { x: 0, y: 0, z: 0 },
                    image_extent: gfx::image::Extent {
                        width,
                        height,
                        depth: 1,
                    },
                }],
            );

            let image_barrier = gfx::memory::Barrier::Image {
                states: (gfx::image::Access::TRANSFER_WRITE, gfx::image::Layout::TransferDstOptimal)
                    ..(gfx::image::Access::SHADER_READ, gfx::image::Layout::ShaderReadOnlyOptimal),
                target: &self.image,
                families: None,
                range: gfx::image::SubresourceRange {
                    aspects: gfx::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                },
            };

            cmd_buffer.pipeline_barrier(
                gfx::pso::PipelineStage::TRANSFER..gfx::pso::PipelineStage::FRAGMENT_SHADER,
                gfx::memory::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();
            cmd_buffer
        };

        unsafe { device.queue_group.queues[0].submit_nosemaphores(std::iter::once(&cmd_buffer), Some(&texture_fence)) };

        // Cleanup staging resources
        unsafe { device.gpu.wait_for_fence(&texture_fence, !0) };
    }
}

impl pipeline::ShaderInput for TextureBuffer {

    fn get_descriptor(&self) -> Option<gfx::pso::Descriptor<Backend>> {
        return Some(gfx::pso::Descriptor::Image(&self.image_view, gfx::image::Layout::Undefined));
    }
    fn get_binding_type(&self) -> gfx::pso::DescriptorType {
        return gfx::pso::DescriptorType::SampledImage;
    }

}