use crate::*;

use gfx::Device as GfxDevice;
use gfx::Swapchain;

use std::iter;

pub type Fence = <Backend as gfx::Backend>::Fence;
pub type Semaphore = <Backend as gfx::Backend>::Semaphore;
pub type CommandPool = gfx::CommandPool<Backend, gfx::Graphics>;

pub struct Frame<'a> {

    pub frame_index: usize,
    pub framebuffer: &'a render::Framebuffer,
    pub command_pool: &'a mut CommandPool,

    pub framebuffer_fence: &'a Fence,
    pub acquire_semaphore: &'a Semaphore,
    pub present_semaphore: &'a Semaphore,

}

impl<'a> Frame<'a> {
    pub fn new(frame_index: usize, framebuffer: &'a render::Framebuffer, command_pool: &'a mut CommandPool, framebuffer_fence: &'a Fence, acquire_semaphore: &'a Semaphore, present_semaphore: &'a Semaphore) -> Self {
        Self { frame_index, framebuffer, command_pool, framebuffer_fence, acquire_semaphore, present_semaphore }
    }

    pub fn begin_render<F>(&mut self, graphics: &mut render::Graphics, mut f: F) -> bool
        where F: FnMut(&mut render::Dispatch) {

        let mut cmd_buffer = CommandBuffer::new(self.command_pool);
        let mut dispatch: render::Dispatch = render::Dispatch::new(graphics, &mut cmd_buffer, self.framebuffer);
        f(&mut dispatch);

        cmd_buffer.finish();

        let submission = gfx::Submission {
            command_buffers: iter::once(&cmd_buffer.cmd),
            wait_semaphores: iter::once((self.acquire_semaphore, gfx::pso::PipelineStage::BOTTOM_OF_PIPE)),
            signal_semaphores: iter::once(self.present_semaphore),
        };

        unsafe { graphics.device.queue_group.queues[0].submit(submission, Some(self.framebuffer_fence)) };

        let result = unsafe { graphics.render_surface.swapchain
            .present(
                &mut graphics.device.queue_group.queues[0],
                self.frame_index as u32,
                Some(self.present_semaphore),
            ) };

        if result.is_err() {
            return false;
        }
        return true;

    }
}

pub struct FrameAggregator {

    pub framebuffers: Vec<render::Framebuffer>,
    pub command_pools: Vec<CommandPool>,

    pub framebuffer_fences: Vec<Fence>,
    pub acquire_semaphores: Vec<Semaphore>,
    pub present_semaphores: Vec<Semaphore>,

    last_ref: usize,

    device_token: core::DeviceToken,

}

impl FrameAggregator {

    pub fn create(render_pass: &render::RenderPass, depth_format: Option<gfx::format::Format>, graphics: &mut render::Graphics) -> Self {
        let framebuffers: Vec<render::Framebuffer> = graphics.render_surface.create_framebuffers(render_pass, depth_format, &graphics.device);
        Self::new(framebuffers, &graphics.device)
    }

    pub fn new(framebuffers: Vec<render::Framebuffer>, device: &core::Device) -> Self {
        let max_buffers = 16;
        let count = framebuffers.len();
        let mut command_pools: Vec<CommandPool> = Vec::with_capacity(count);
        let mut framebuffer_fences: Vec<Fence> = Vec::with_capacity(count);
        let mut acquire_semaphores: Vec<Semaphore> = Vec::with_capacity(count);
        let mut present_semaphores: Vec<Semaphore> = Vec::with_capacity(count);
        for i in 0..count {

            let command_pool = unsafe {  device.gpu.create_command_pool_typed(
                &device.queue_group,
                gfx::pool::CommandPoolCreateFlags::empty()
            ).unwrap() };
            command_pools.push(command_pool);
            framebuffer_fences.push(device.gpu.create_fence(true).unwrap());
            acquire_semaphores.push(device.gpu.create_semaphore().unwrap());
            present_semaphores.push(device.gpu.create_semaphore().unwrap());

        }

        return Self { framebuffers, command_pools, framebuffer_fences, acquire_semaphores, present_semaphores, last_ref: 0, device_token: device.create_token() };
    }

    fn next_acq_pre_pair_index(&mut self) -> usize {
        if self.last_ref >= self.acquire_semaphores.len() {
            self.last_ref = 0
        }

        let ret = self.last_ref;
        self.last_ref += 1;
        ret
    }

    fn acquire_next_indices(&mut self, graphics: &mut render::Graphics) -> Option<(usize, usize)> {
        if !self.acquire_semaphores.is_empty() {
            let si = self.next_acq_pre_pair_index();
            if let Ok(fi) = unsafe { graphics.render_surface.swapchain.acquire_image(!0, gfx::FrameSync::Semaphore(&self.acquire_semaphores[si])) } {
                return Some((fi as usize, si));
            }
        }
        None
    }

    pub fn acquire_next(&mut self, graphics: &mut render::Graphics) -> Option<Frame> {
        if let Some((fi, si)) = self.acquire_next_indices(graphics) {
            return Some(self.next_frame(fi, si));
        }
        None
    }

    pub fn next_frame(&mut self, frame_index: usize, semaphore_index: usize) -> Frame {
        Frame::new(frame_index, &self.framebuffers[frame_index], &mut self.command_pools[frame_index], &self.framebuffer_fences[frame_index], &self.acquire_semaphores[semaphore_index], &self.present_semaphores[semaphore_index])
    }

}

impl Drop for FrameAggregator {

    fn drop(&mut self) {
        unsafe {
            use std::mem;
            for framebuffer in self.framebuffers.iter() {
                self.device_token.gpu.destroy_framebuffer(mem::transmute_copy(framebuffer));
            }
            for command_pool in self.command_pools.iter() {
                self.device_token.gpu.destroy_command_pool(mem::transmute_copy(command_pool));
            }
            for fence in self.framebuffer_fences.iter() {
                self.device_token.gpu.destroy_fence(mem::transmute_copy(fence));
            }
            for acquire_semaphore in self.acquire_semaphores.iter() {
                self.device_token.gpu.destroy_semaphore(mem::transmute_copy(acquire_semaphore));
            }
            for present_semaphore in self.present_semaphores.iter() {
                self.device_token.gpu.destroy_semaphore(mem::transmute_copy(present_semaphore));
            }
        }
    }

}

pub struct Encoder<'a> {
    pub pass: gfx::command::RenderPassInlineEncoder<'a, Backend>,
}

impl<'a> Encoder<'a> {

    pub fn new(pass: gfx::command::RenderPassInlineEncoder<'a, Backend>) -> Encoder {
        return Encoder { pass };
    }

}

pub struct CommandBuffer {

    pub cmd: gfx::command::CommandBuffer<Backend, gfx::Graphics>,

}

impl CommandBuffer {

    pub fn new(command_pool: &mut gfx::CommandPool<Backend, gfx::Graphics>) -> Self {

        let cmd = command_pool.acquire_command_buffer::<gfx::command::OneShot>();

        return CommandBuffer { cmd };

    }

    pub fn begin_draw(&mut self, framebuffer: &render::Framebuffer, render_pass: &render::RenderPass, render_surface: &render::Surface, clear_color: Color) -> Encoder {
        unsafe {
            self.cmd.set_viewports(0, &[render_surface.viewport.clone()]);

            self.cmd.set_scissors(0, &[render_surface.viewport.clone().rect]);

            self.cmd.set_depth_bounds(0.0..1.0);

            let encoder = self.cmd.begin_render_pass_inline(
                &render_pass.raw_render_pass,
                &framebuffer,
                render_surface.viewport.rect,
                &[
                    gfx::command::ClearValue::Color(gfx::command::ClearColor::Float(clear_color.to_raw_color())),
                    gfx::command::ClearValue::DepthStencil(gfx::command::ClearDepthStencil(1.0, 0)),
                ],
            );
            return Encoder::new(encoder);
        }
    }

    pub fn finish(&mut self) {
        unsafe { self.cmd.finish() };
    }

}
