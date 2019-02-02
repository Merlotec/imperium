use crate::*;



pub struct SpatialPass {

    pub pass: render::RenderPass,
    pub frame_aggregator: command::FrameAggregator,

}

impl SpatialPass {

    pub fn new(graphics: &mut render::Graphics) -> Self {

        let pass: render::RenderPass = render::RenderPass::create_basic(&graphics.device);
        let frame_aggregator = command::FrameAggregator::create(&pass, Some(render::RenderPass::STD_DEPTH_FORMAT), graphics);
        return Self { pass, frame_aggregator };

    }

    pub fn next(&mut self, graphics: &mut render::Graphics) -> Option<(command::Frame, &render::RenderPass)> {
        if graphics.render_surface.did_rebuild {
            self.frame_aggregator = command::FrameAggregator::create(&self.pass, Some(render::RenderPass::STD_DEPTH_FORMAT), graphics);
        }
        if let Some(frame) = self.frame_aggregator.acquire_next(graphics) {
            // Invalidate swapchain for rebuilding.
            return Some((frame, &self.pass));
        } else {
            graphics.render_surface.invalidate();

        }
        None

    }

}