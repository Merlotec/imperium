use super::*;

use std::io::Read;
use std::mem;
use backend::Backend;
use gfx::PhysicalDevice;
use gfx::Instance;
use gfx::Surface as GfxSurface;
use gfx::Device as GfxDevice;
use gfx::Swapchain;
use gfx::DescriptorPool;

const DEPTH_FORMAT: gfx::format::Format = gfx::format::Format::D32FloatS8Uint;

pub struct Surface {

    pub window_surface: window::WindowSurface,

    pub swapchain: <Backend as gfx::Backend>::Swapchain,
    pub framebuffers: Vec<<Backend as gfx::Backend>::Framebuffer>,
    pub images: Vec<<Backend as gfx::Backend>::ImageView>,

    pub depth_view: DepthView,

    pub viewport: gfx::pso::Viewport,

}

impl Surface {

    pub fn create(mut window_surface: window::WindowSurface, device: &mut core::Device, window: &window::Window, render_pass: &RenderPass) -> Surface {

        let swap_config = gfx::SwapchainConfig::from_caps(&device.capabilites, device.color_format);

        let extent = swap_config.extent.to_extent();

        let viewport = gfx::pso::Viewport {
            rect: gfx::pso::Rect {
                x: 0 as i16,
                y: 0 as i16,
                w: extent.width as i16,
                h: extent.height as i16,
            },
            depth: (0.0 as f32)..(1.0 as f32),
        };

        let (swapchain, backbuffer) = device.device.create_swapchain(&mut window_surface.surface, swap_config, None);

        let depth_view: DepthView = DepthView::create(&device, Vector2f::new(extent.width as f32, extent.height as f32));

        let (images, framebuffers) = match backbuffer {
            gfx::Backbuffer::Images(images) => {
                let color_range = gfx::image::SubresourceRange {
                    aspects: gfx::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                };

                let image_views = images
                    .iter()
                    .map(|image| {
                        device.device
                            .create_image_view(
                                image,
                                gfx::image::ViewKind::D2,
                                device.color_format,
                                gfx::format::Swizzle::NO,
                                color_range.clone(),
                            ).unwrap()
                    }).collect::<Vec<_>>();

                let fbos = image_views
                    .iter()
                    .map(|image_view| {
                        device.device
                            .create_framebuffer(&render_pass.raw_render_pass, vec![image_view, &depth_view.view], extent)
                            .unwrap()
                    }).collect();

                (image_views, fbos)
            }

            // This arm of the branch is currently only used by the OpenGL backend,
            // which supplies an opaque framebuffer for you instead of giving you control
            // over individual images.
            gfx::Backbuffer::Framebuffer(fbo) => (vec![], vec![fbo]),
        };

        return Surface { window_surface, swapchain, images, framebuffers, viewport, depth_view }

    }

    /// Rebuilds the swapchain data for this surface object.
    pub fn rebuild(&mut self, device: &mut core::Device, window: &window::Window, render_pass: &RenderPass, command_dispatch: &mut command::CommandDispatch) {
        self.destroy(device, command_dispatch);
        let (caps, _, _) = self.window_surface.surface.compatibility(&device.adapter.physical_device);
        let swap_config = gfx::SwapchainConfig::from_caps(&caps, device.color_format);
        let extent = swap_config.extent.to_extent();

        let viewport = gfx::pso::Viewport {
            rect: gfx::pso::Rect {
                x: 0 as i16,
                y: 0 as i16,
                w: extent.width as i16,
                h: extent.height as i16,
            },
            depth: (0.0 as f32)..(1.0 as f32),
        };

        // Here we just create the swapchain, image views, and framebuffers
        // like we did in part 00, and store them in swapchain_stuff.
        let swap_config = gfx::SwapchainConfig::from_caps(&caps, device.color_format);
        let extent = swap_config.extent.to_extent();
        let (swapchain, backbuffer) =  device.device.create_swapchain(&mut self.window_surface.surface, swap_config, None);

        let depth_view: DepthView = DepthView::create(&device, Vector2f::new(extent.width as f32, extent.height as f32));

        let (frame_views, framebuffers) = match backbuffer {
            gfx::Backbuffer::Images(images) => {
                let color_range = gfx::image::SubresourceRange {
                    aspects: gfx::format::Aspects::COLOR,
                    levels: 0..1,
                    layers: 0..1,
                };

                let image_views = images
                    .iter()
                    .map(|image| {
                        device.device
                            .create_image_view(
                                image,
                                gfx::image::ViewKind::D2,
                                device.color_format,
                                gfx::format::Swizzle::NO,
                                color_range.clone(),
                            ).unwrap()
                    }).collect::<Vec<_>>();

                let fbos = image_views
                    .iter()
                    .map(|image_view| {
                        device.device
                            .create_framebuffer(&render_pass.raw_render_pass, vec![image_view, &depth_view.view], extent)
                            .unwrap()
                    }).collect();

                (image_views, fbos)
            }
            gfx::Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
        };

        // Store the new stuff.
       // swapchain_stuff = Some((swapchain, extent, frame_views, framebuffers));
        self.swapchain = swapchain;
        self.images = frame_views;
        self.framebuffers = framebuffers;
        self.depth_view = depth_view;
        self.viewport = viewport;
    }

    pub fn destroy(&mut self, device: &core::Device, command_dispatch: &mut command::CommandDispatch) {
        // We want to wait for all queues to be idle and reset the command pool,
        // so that we know that no commands are being executed while we destroy
        // the swapchain.
        device.device.wait_idle().expect("Failed to wait idle device!");
        command_dispatch.command_pool.reset();

        // Destroy all the old stuff.
        for framebuffer in self.framebuffers.iter() {
            device.device.destroy_framebuffer( unsafe { mem::transmute_copy(framebuffer) });
        }

        for image_view in self.images.iter() {
            device.device.destroy_image_view(unsafe { mem::transmute_copy(image_view) });
        }

        device.device.destroy_swapchain(unsafe { mem::transmute_copy(&self.swapchain) });
    }

    pub fn next_index(&mut self, command_dispatch: &command::CommandDispatch) -> Option<gfx::SwapImageIndex> {
        return self.swapchain.acquire_image(!0, gfx::FrameSync::Semaphore(&command_dispatch.frame_semaphore)).ok();
    }

    pub fn get_size(&self) -> Vector2f {
        return Vector2f::new(self.viewport.rect.w as f32, self.viewport.rect.h as f32);
    }

}

pub struct RenderPass {

    pub raw_render_pass: <Backend as gfx::Backend>::RenderPass,

}

impl RenderPass {

    pub fn create(device: &core::Device) -> Self {

        let raw_render_pass = {

            let color_attachment = gfx::pass::Attachment {
                format: Some(device.color_format),
                samples: 1,
                ops: gfx::pass::AttachmentOps::new(gfx::pass::AttachmentLoadOp::Clear, gfx::pass::AttachmentStoreOp::Store),
                stencil_ops: gfx::pass::AttachmentOps::DONT_CARE,
                layouts: gfx::image::Layout::Undefined..gfx::image::Layout::Present,
            };

            let depth_attachment = gfx::pass::Attachment {
                format: Some(DEPTH_FORMAT),
                samples: 1,
                ops: gfx::pass::AttachmentOps::new(gfx::pass::AttachmentLoadOp::Clear, gfx::pass::AttachmentStoreOp::DontCare),
                stencil_ops: gfx::pass::AttachmentOps::DONT_CARE,
                layouts: gfx::image::Layout::Undefined..gfx::image::Layout::DepthStencilAttachmentOptimal,
            };

            let subpass = gfx::pass::SubpassDesc {
                colors: &[(0, gfx::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: Some(&(1, gfx::image::Layout::DepthStencilAttachmentOptimal)),
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            let dependency = gfx::pass::SubpassDependency {
                passes: gfx::pass::SubpassRef::External..gfx::pass::SubpassRef::Pass(0),
                stages: gfx::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT..gfx::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                accesses: gfx::image::Access::empty()
                    ..(gfx::image::Access::COLOR_ATTACHMENT_READ | gfx::image::Access::COLOR_ATTACHMENT_WRITE),
            };

            device.device.create_render_pass(&[color_attachment, depth_attachment], &[subpass], &[dependency])

        };

        return Self { raw_render_pass,  };

    }

}

/// The renderer structure contains all graphics data for the engine.
/// This structure owns the device object.
/// It also contains the 'Surface' object and the 'RenderPass' object.
/// NOTE: This structure does not contain command buffer data, only pure graphics data like the device and the surfaces.
pub struct Graphics {

    pub device: core::Device,
    pub render_surface: Surface,
    pub render_pass: RenderPass,

}

impl Graphics {

    /// Creates a new renderer object from the specified instance and window.
    pub fn create(instance: &core::Instance, window: &window::Window) -> Self {

        let mut window_surface: window::WindowSurface = window::WindowSurface::create(instance, window);
        let mut device: core::Device = core::Device::create(instance, &window_surface);
        let render_pass: RenderPass = RenderPass::create(&device);
        let render_surface: Surface = Surface::create(window_surface, &mut device, window, &render_pass);

        return Self { device, render_pass, render_surface };

    }
}

/// Contains all graphics data neccessary for rendering.
/// This includes the command buffer object.
pub struct Renderer {

    pub graphics: Graphics,
    pub command_dispatch: command::CommandDispatch,

}

impl Renderer {

    pub fn create(instance: &core::Instance, window: &window::Window) -> Self {

        let graphics: Graphics = Graphics::create(instance, window);

        let command_dispatch: command::CommandDispatch = command::CommandDispatch::create(&graphics.device);

        return Self { graphics, command_dispatch };
    }

}

/// The structure which contains depth image information for rendering depth properly.
pub struct DepthView {

    pub image: <Backend as gfx::Backend>::Image,
    pub view: <Backend as gfx::Backend>::ImageView,
    pub memory: <Backend as gfx::Backend>::Memory,

}

impl DepthView {

    pub fn create(device: &core::Device, size: Vector2f) -> Self {

        let kind = gfx::image::Kind::D2(size.x as gfx::image::Size, size.y as gfx::image::Size, 1, 1);

        let memory_types = device.adapter.physical_device.memory_properties().memory_types;

        let unbound_depth_image = device.device
            .create_image(
                kind,
                1,
                DEPTH_FORMAT,
                gfx::image::Tiling::Optimal,
                gfx::image::Usage::DEPTH_STENCIL_ATTACHMENT,
                gfx::image::ViewCapabilities::empty(),
            ).log_expect("Failed to create unbound depth image");

        let image_req = device.device.get_image_requirements(&unbound_depth_image);

        let device_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, memory_type)| {
                image_req.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(gfx::memory::Properties::DEVICE_LOCAL)
            }).log_expect("Failed to find device memory type")
            .into();

        let memory = device.device
            .allocate_memory(device_type, image_req.size)
            .log_expect("Failed to allocate depth image");

        let image = device.device
            .bind_image_memory(&memory, 0, unbound_depth_image)
            .log_expect("Failed to bind depth image");

        let view = device.device
            .create_image_view(
                &image,
                gfx::image::ViewKind::D2,
                DEPTH_FORMAT,
                gfx::format::Swizzle::NO,
                gfx::image::SubresourceRange {
                    aspects: gfx::format::Aspects::DEPTH | gfx::format::Aspects::STENCIL,
                    levels: 0..1,
                    layers: 0..1,
                },
            ).log_expect("Failed to create image view");
        return Self { image, view, memory };
    }

}

#[derive(Copy, Clone)]
pub struct RenderTransform {

    pub model: Matrix4f,
    pub view: Matrix4f,
    pub projection: Matrix4f,

}

impl RenderTransform {

    pub fn identity() -> RenderTransform {

        return RenderTransform { model: Matrix4f::identity(), view: Matrix4f::identity(), projection: Matrix4f::identity() };

    }

    pub fn new(model: Matrix4f, view: Matrix4f, projection: Matrix4f) -> RenderTransform {
        return RenderTransform { model, view, projection };
    }

    pub fn create_buffer(self, device: &core::Device) -> buffer::Buffer {
        return buffer::Buffer::create_uniform(&[self], device);
    }

}
