use crate::*;

use gfx::Device as GfxDevice;
use gfx::Swapchain;

pub struct CommandDispatch {

    pub command_pool: gfx::CommandPool<Backend, gfx::Graphics>,

    pub frame_semaphore: <Backend as gfx::Backend>::Semaphore,
    pub present_semaphore: <Backend as gfx::Backend>::Semaphore,

}

impl CommandDispatch {

    pub fn create(device: &core::Device) -> CommandDispatch {
        let max_buffers = 16;
        let command_pool = device.device.create_command_pool_typed(
            &device.queue_group,
            gfx::pool::CommandPoolCreateFlags::empty(),
            max_buffers,
        );

        let frame_semaphore = device.device.create_semaphore();
        let present_semaphore = device.device.create_semaphore();

        return CommandDispatch { command_pool, frame_semaphore, present_semaphore };
    }

    pub fn reset(&mut self) {
        self.command_pool.reset();
    }

    pub fn dispatch_render<F>(&mut self, clear_color: Color, graphics: &mut render::Graphics, mut exec: F) -> bool
        where F: FnMut(&mut render::Graphics, &mut Encoder) {
        self.reset();
        let frame_index = match graphics.render_surface.next_index(self) {
            Some(v) => v,
            None => return true,
        };
        let mut command_buffer = CommandBuffer::new(&mut self.command_pool, frame_index as usize);
        {
            let mut encoder = command_buffer.begin_draw(&graphics.render_pass, &graphics.render_surface, clear_color);
            {
                exec(graphics, &mut encoder);
            }
        }

        let submission = gfx::Submission::new()
            .wait_on(&[(&self.frame_semaphore, gfx::pso::PipelineStage::BOTTOM_OF_PIPE)])
            .signal(&[&self.present_semaphore])
            .submit(vec![command_buffer.finish()]);

        // We submit the submission to one of our command queues, which will signal
        // frame_fence once rendering is completed.
        graphics.device.queue_group.queues[0].submit(submission, None);

        // We first wait for the rendering to complete...
        // TODO: Fix up for semaphores

        // ...and then present the image on screen!
        let result = graphics.render_surface.swapchain
            .present(
                &mut graphics.device.queue_group.queues[0],
                frame_index,
                vec![&self.present_semaphore],
            );
        if result.is_err() {
            return false;
        }
        return true;
    }

}

pub struct Encoder<'a> {
    pub pass: gfx::command::RenderPassInlineEncoder<'a, Backend, gfx::command::Primary>,
}

impl<'a> Encoder<'a> {

    pub fn new(pass: gfx::command::RenderPassInlineEncoder<'a, Backend, gfx::command::Primary>) -> Encoder {
        return Encoder { pass };
    }

}

pub struct CommandBuffer<'a> {

    pub cmd: gfx::command::CommandBuffer<'a, Backend, gfx::Graphics>,
    pub frame_index: usize,

}

impl<'a> CommandBuffer<'a> {

    pub fn new(command_pool: &'a mut gfx::CommandPool<Backend, gfx::Graphics>, frame_index: usize) -> Self {

        let cmd = command_pool.acquire_command_buffer(false);

        return CommandBuffer { cmd, frame_index };

    }

    pub fn begin_draw(&mut self, render_pass: &render::RenderPass, render_surface: &render::Surface, clear_color: Color) -> Encoder {

        self.cmd.set_viewports(0, &[render_surface.viewport.clone()]);

        self.cmd.set_scissors(0, &[render_surface.viewport.clone().rect]);

        self.cmd.set_depth_bounds(0.0..1.0);

        let encoder = self.cmd.begin_render_pass_inline(
            &render_pass.raw_render_pass,
            &render_surface.framebuffers[self.frame_index],
            render_surface.viewport.rect,
            &[
                gfx::command::ClearValue::Color(gfx::command::ClearColor::Float(clear_color.to_raw_color())),
                gfx::command::ClearValue::DepthStencil(gfx::command::ClearDepthStencil(1.0, 0)),
            ],
        );

        return Encoder::new(encoder);

    }

    pub fn finish(self) -> gfx::command::Submit<backend::Backend, gfx::Graphics, gfx::command::OneShot, gfx::command::Primary> {
        return self.cmd.finish();
    }

}
