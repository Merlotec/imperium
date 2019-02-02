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



pub type Framebuffer = <Backend as gfx::Backend>::Framebuffer;

pub struct Surface {

    pub is_valid: bool,
    pub did_rebuild: bool,

    pub window_surface: window::WindowSurface,

    pub swapchain: <Backend as gfx::Backend>::Swapchain,
    pub backbuffer: Option<gfx::Backbuffer<Backend>>,

    pub viewport: gfx::pso::Viewport,
    pub extent: gfx::image::Extent,

    device_token: core::DeviceToken,

}

impl Surface {

    pub fn create(mut window_surface: window::WindowSurface, device: &mut core::Device) -> Surface {

        let swap_config = gfx::SwapchainConfig::from_caps(&device.capabilites, device.color_format, gfx::window::Extent2D { width: window_surface.size.x as u32, height: window_surface.size.y as u32 });

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

        let (swapchain, backbuffer) = unsafe { device.gpu.create_swapchain(&mut window_surface.surface, swap_config, None).unwrap() };

        return Surface { is_valid: true, did_rebuild: false, window_surface, swapchain, backbuffer: Some(backbuffer), viewport, extent, device_token: device.create_token() }

    }

    pub fn create_framebuffers(&mut self, render_pass: &RenderPass, depth: Option<gfx::format::Format>, device: &core::Device) -> Vec<Framebuffer> {

        let mut depth_view: Option<buffer::TextureBuffer> = None;

        if let Some(depth_format) = depth {
            depth_view = Some(buffer::TextureBuffer::create_depth(Vector2u::new(self.extent.width, self.extent.height), depth_format, &device));
        }

        unsafe {
            if let Some(backbuffer) = self.backbuffer.take() {
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
                                device.gpu
                                    .create_image_view(
                                        image,
                                        gfx::image::ViewKind::D2,
                                        device.color_format,
                                        gfx::format::Swizzle::NO,
                                        color_range.clone(),
                                    ).unwrap()
                            }).collect::<Vec<_>>();

                        let fbos: Vec<Framebuffer> = image_views
                            .iter()
                            .map(|image_view| {
                                let attachments: Vec<&<Backend as gfx::Backend>::ImageView>;
                                if let Some(depth) = depth_view.as_ref() {
                                    attachments = vec![image_view, &depth.image_view];
                                } else {
                                    attachments = vec![image_view];
                                }
                                device.gpu
                                    .create_framebuffer(&render_pass.raw_render_pass, attachments, self.extent)
                                    .unwrap()
                            }).collect();

                        (image_views, fbos)
                    }
                    gfx::Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
                };
                return framebuffers;
            }
        }
        return Vec::new();
    }

    /// Makes the swapchain invalid so that we must rebuild it next frame.
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Rebuilds the swapchain data for this surface object.
    pub fn rebuild(&mut self, window: &window::Window, device: &mut core::Device) {
        self.destroy_swapchain();
        self.window_surface.size = window.get_size();
        let (caps, _, _, _) = self.window_surface.surface.compatibility(&device.adapter.physical_device);
        let swap_config = gfx::SwapchainConfig::from_caps(&caps, device.color_format, gfx::window::Extent2D { width: self.window_surface.size.x as u32, height: self.window_surface.size.y as u32 });
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

        // We can use `transmute_copy` because we will not use it again.
        let (swapchain, backbuffer) =  unsafe { device.gpu.create_swapchain(&mut self.window_surface.surface, swap_config, Some(mem::transmute_copy(&self.swapchain))).unwrap() };

        // Store the new stuff.
       // swapchain_stuff = Some((swapchain, extent, frame_views, framebuffers));
        self.swapchain = swapchain;
        self.backbuffer = Some(backbuffer);
        self.viewport = viewport;
        self.extent = extent;

        // Revalidate.
        self.is_valid = true;
        self.did_rebuild = true;
    }

    pub fn destroy_swapchain(&mut self) {
        self.device_token.gpu.wait_idle().expect("Failed to wait idle device!");
    }

    pub fn get_size(&self) -> Vector2f {
        return Vector2f::new(self.viewport.rect.w as f32, self.viewport.rect.h as f32);
    }

}

impl Drop for Surface {

    fn drop(&mut self) {
        unsafe { self.device_token.gpu.destroy_swapchain(mem::transmute_copy(&self.swapchain) ) };
    }

}

pub struct RenderPass {

    pub raw_render_pass: <Backend as gfx::Backend>::RenderPass,

}

impl RenderPass {

    pub const STD_DEPTH_FORMAT: gfx::format::Format = gfx::format::Format::D32FloatS8Uint;

    pub fn create_basic(device: &core::Device) -> Self {

        let raw_render_pass = {

            let color_attachment = gfx::pass::Attachment {
                format: Some(device.color_format),
                samples: 1,
                ops: gfx::pass::AttachmentOps::new(gfx::pass::AttachmentLoadOp::Clear, gfx::pass::AttachmentStoreOp::Store),
                stencil_ops: gfx::pass::AttachmentOps::DONT_CARE,
                layouts: gfx::image::Layout::Undefined..gfx::image::Layout::Present,
            };

            let depth_attachment = gfx::pass::Attachment {
                format: Some(Self::STD_DEPTH_FORMAT),
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

            unsafe { device.gpu.create_render_pass(&[color_attachment, depth_attachment], &[subpass], &[dependency]).unwrap() }

        };

        return Self { raw_render_pass,  };

    }

    pub fn from_raw(raw: <Backend as gfx::Backend>::RenderPass) -> Self {
        return Self { raw_render_pass: raw };
    }

}

/// The renderer structure contains all graphics data for the engine.
/// This structure owns the device object.
/// It also contains the 'Surface' object and the 'RenderPass' object.
/// NOTE: This structure does not contain command buffer data, only pure graphics data like the device and the surfaces.
pub struct Graphics {

    pub device: core::Device,
    pub render_surface: Surface,

}

impl Graphics {

    /// Creates a new renderer object from the specified instance and window.
    pub fn create(instance: &core::Instance, window: &window::Window) -> Self {

        let mut window_surface: window::WindowSurface = window::WindowSurface::create(instance, window);
        let mut device: core::Device = core::Device::create(instance, &window_surface);
        let render_surface: Surface = Surface::create(window_surface, &mut device);

        return Self { device, render_surface };

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

    pub fn alloc_buffer(self, device: &core::Device) -> buffer::Buffer {
        return buffer::Buffer::alloc_uniform(&[self], device);
    }

}

pub struct Dispatch<'a> {
    pub graphics: &'a mut Graphics,
    // We need to use a raw pointer as to avoid lifetimes. (I know... it's exciting)
    pub command_buffer: &'a mut command::CommandBuffer,

    pub framebuffer: &'a Framebuffer,
}

impl<'a> Dispatch<'a> {
    pub fn new(graphics: &'a mut Graphics, command_buffer: &'a mut command::CommandBuffer, framebuffer: &'a Framebuffer) -> Self {
        return Self { graphics, command_buffer, framebuffer };
    }

    pub fn begin_render_pass_inline<F>(&mut self, clear_color: Color, render_pass: &RenderPass, mut f: F)
    where F: FnMut(&mut Graphics, &mut command::Encoder) {
        let mut encoder = self.command_buffer.begin_draw(&self.framebuffer, render_pass, &self.graphics.render_surface, clear_color);
        f(&mut self.graphics, &mut encoder);
    }
}

pub struct DispatchUnsafe {
    graphics: *mut Graphics,
    // We need to use a raw pointer as to avoid lifetimes. (I know... it's exciting)
    command_buffer: *mut command::CommandBuffer,
    framebuffer: *const Framebuffer,
}

impl DispatchUnsafe {
    /// FOR THIS TO BE USED SAFELY, ONLY ONE BORROW OF GRAPHICS AND ENCODER SHOULD BE DONE AT ONCE, AND THIS OBJECT SHOULD NOT LIVE LONGER THAN THE RENDER CYCLE!!!
    pub fn new(graphics: *mut Graphics, command_buffer: *mut command::CommandBuffer, framebuffer: *const Framebuffer) -> Self {
        return Self { graphics, command_buffer, framebuffer };
    }

    /// This is suicide...
    pub unsafe fn make_safe(&mut self) -> &mut Dispatch {
        return &mut *((self as *mut DispatchUnsafe) as *mut Dispatch);
    }

    pub unsafe fn graphics(&self) -> &Graphics {
        return &*self.graphics;
    }

    /// Although it should be used in a safe context, we will mark it as unsafe purely due to the nature of these operations.
    /// We're even following the mutability rules here!
    pub unsafe fn graphics_mut(&mut self) -> &mut Graphics {
        return &mut *self.graphics;
    }

    pub unsafe fn command_buffer(&self) -> &command::CommandBuffer {
        return &*self.command_buffer;
    }

    pub unsafe fn command_bufferr_mut(&mut self) -> &mut command::CommandBuffer {
        // We are downgrading to const.
        return &mut *self.command_buffer;
    }
}

unsafe impl Send for DispatchUnsafe {}
unsafe impl Sync for DispatchUnsafe {}