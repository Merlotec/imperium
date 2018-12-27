use crate::*;

use std::marker::PhantomData;

use gfx::Device as GfxDevice;
use gfx::PhysicalDevice;

/// This structure represents a GPU buffer which contains memory of a specific type.
/// This holds both the buffer object and the buffer memory.
pub struct Buffer {

    pub buf: <Backend as gfx::Backend>::Buffer,
    pub memory: <Backend as gfx::Backend>::Memory,
    pub count: usize,

    device_token: core::DeviceToken,

}

impl Buffer {

    pub fn create_vertex<T: std::marker::Copy>(slice: &[T], device: &core::Device) -> Self {
        return Self::create(slice, gfx::buffer::Usage::VERTEX, gfx::memory::Properties::CPU_VISIBLE, device);
    }

    pub fn create_uniform<T: std::marker::Copy>(slice: &[T], device: &core::Device) -> Self {
        return Self::create(slice, gfx::buffer::Usage::UNIFORM, gfx::memory::Properties::CPU_VISIBLE, device);
    }

    pub fn create<T: std::marker::Copy>(slice: &[T], usage: gfx::buffer::Usage, properties: gfx::memory::Properties, device: &core::Device) -> Self {
        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let stride = std::mem::size_of::<T>() as u64;
        let buffer_len = slice.len() as u64 * stride;
        let unbound_buffer = device.device
            .create_buffer(buffer_len, usage)
            .unwrap();


        let (upload_type, req) = device.upload_type_for(&unbound_buffer, properties);

        let buffer_memory = device.device.allocate_memory(upload_type, req.size).unwrap();

        let buffer = device.device
            .bind_buffer_memory(&buffer_memory, 0, unbound_buffer)
            .unwrap();

        {
            let mut dest = device.device
                .acquire_mapping_writer::<T>(&buffer_memory, 0..buffer_len)
                .unwrap();
            dest.copy_from_slice(slice);
            device.device.release_mapping_writer(dest);
        }

        return Self { buf: buffer, memory: buffer_memory, count: slice.len(), device_token: device.create_token() };
    }

    pub fn create_empty<T: std::marker::Copy>(count: usize, usage: gfx::buffer::Usage, properties: gfx::memory::Properties, device: &core::Device) -> Self {

        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let stride = std::mem::size_of::<T>() as u64;
        let buffer_len = count as u64 * stride;
        let unbound_buffer = device.device.create_buffer(buffer_len, usage).unwrap();
        let req = device.device.get_buffer_requirements(&unbound_buffer);
        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, ty)| req.type_mask & (1 << id) != 0 && ty.properties.contains(properties))
            .unwrap()
            .into();

        let buffer_memory = device.device.allocate_memory(upload_type, req.size).unwrap();
        let buffer = device.device
            .bind_buffer_memory(&buffer_memory, 0, unbound_buffer)
            .unwrap();

        return Self { buf: buffer, memory: buffer_memory, count, device_token: device.create_token() };

    }

    /// It is the responsibility of the programmer to ensure the data is of the right size and format.
    pub fn fill_buffer<T: Copy>(&self, data: &[T], device: &core::Device) {
        let stride = std::mem::size_of::<T>() as u64;
        let buffer_len = data.len() as u64 * stride;
        let mut dest = device.device.acquire_mapping_writer::<T>(&self.memory, 0..buffer_len).unwrap();

        dest.copy_from_slice(data);

        device.device.release_mapping_writer(dest);
    }

}

impl std::ops::Drop for Buffer {

    fn drop(&mut self) {
        unsafe {
            use std::mem;
            self.device_token.device.destroy_buffer(mem::transmute_copy(&mut self.buf));
            self.device_token.device.free_memory(mem::transmute_copy(&mut self.memory));
        }
    }

}

pub struct TextureBuffer {

    pub image: <Backend as gfx::Backend>::Image,
    pub memory: <Backend as gfx::Backend>::Memory,
    pub image_view: <Backend as gfx::Backend>::ImageView,

}

impl TextureBuffer {

    pub fn create_image(device: &core::Device, size: Vector2u, format: gfx::format::Format, usage: gfx::image::Usage, aspects: gfx::format::Aspects) -> (<Backend as gfx::Backend>::Image, <Backend as gfx::Backend>::Memory, <Backend as gfx::Backend>::ImageView) {

        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let kind = gfx::image::Kind::D2(size.x, size.y, 1, 1);

        let unbound_image = device.device
            .create_image(
                kind,
                1,
                format,
                gfx::image::Tiling::Optimal,
                usage,
                gfx::image::ViewCapabilities::empty(),
            ).expect("Failed to create unbound image");

        let image_req = device.device.get_image_requirements(&unbound_image);

        let device_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                image_req.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(gfx::memory::Properties::DEVICE_LOCAL)
            }).unwrap()
            .into();

        let image_memory = device.device
            .allocate_memory(device_type, image_req.size)
            .expect("Failed to allocate image");

        let image = device.device
            .bind_image_memory(&image_memory, 0, unbound_image)
            .expect("Failed to bind image");

        let image_view = device.device
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

        return (image, image_memory, image_view);

    }

    pub fn create(texture: &texture::Texture, device: &mut core::Device, command_dispatch: &mut command::CommandDispatch) -> TextureBuffer {

        let texture_fence = device.device.create_fence(false);

        let (texture_image, texture_memory, texture_view) = {
            let (width, height) = (texture.dimensions.x, texture.dimensions.y);

            let (texture_image, texture_memory, texture_view) = Self::create_image(
                &device,
                Vector2u::new(width, height),
                gfx::format::Format::Rgba8Srgb,
                gfx::image::Usage::TRANSFER_DST | gfx::image::Usage::SAMPLED,
                gfx::format::Aspects::COLOR,
            );


            {
                let row_alignment_mask = device.adapter.physical_device.limits().min_buffer_copy_pitch_alignment as u32 - 1;
                let image_stride = 4usize;
                let row_pitch =
                    (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
                let upload_size = u64::from(height * row_pitch);

                let upload_buffer = Buffer::create_empty::<u8>(
                    upload_size as usize,
                    gfx::buffer::Usage::TRANSFER_SRC,
                    gfx::memory::Properties::CPU_VISIBLE,
                    &device
                );

                {
                    let mut data = device.device
                        .acquire_mapping_writer::<u8>(&upload_buffer.memory, 0..upload_size)
                        .unwrap();

                    for y in 0..height as usize {
                        let row = &(*texture.data)[y * (width as usize) * image_stride
                            ..(y + 1) * (width as usize) * image_stride];
                        let dest_base = y * row_pitch as usize;
                        data[dest_base..dest_base + row.len()].copy_from_slice(row);
                    }

                    device.device.release_mapping_writer(data);
                }

                let submit = {
                    let mut cmd_buffer = command_dispatch.command_pool.acquire_command_buffer(false);

                    let image_barrier = gfx::memory::Barrier::Image {
                        states: (gfx::image::Access::empty(), gfx::image::Layout::Undefined)
                            ..(gfx::image::Access::TRANSFER_WRITE, gfx::image::Layout::TransferDstOptimal),
                        target: &texture_image,
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
                        &texture_image,
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
                        target: &texture_image,
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

                    cmd_buffer.finish()
                };

                let submission = gfx::queue::Submission::new().submit(Some(submit));

                device.queue_group.queues[0].submit(submission, Some(&texture_fence));

                // Cleanup staging resources
                device.device.wait_for_fence(&texture_fence, !0);
            }

            (texture_image, texture_memory, texture_view)
        };

        return TextureBuffer { image: texture_image, memory: texture_memory, image_view: texture_view };
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