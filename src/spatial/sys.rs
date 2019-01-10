use crate::*;
use node::Node3D;
use spatial::pipe::mesh::*;
use spatial::model::MeshComponent;
use spatial::RenderComponent;
use scene::*;

pub struct MeshRenderSystem;

impl<'a> System<'a> for MeshRenderSystem {

    type SystemData = (
        WriteExpect<'a, Option<render::RenderCoreUnsafe>>,
        ReadExpect<'a, SceneData>,
        WriteExpect<'a, MeshRenderPipeline>,
        WriteStorage<'a, MeshComponent>,
        ReadStorage<'a, node::NodeObject3D>,
    );

    fn run(&mut self, (mut render_core_unsafe, scene_data, mut mesh_pipeline, mut mesh_components, nodes): Self::SystemData) {
        // Only render if the render core is valid.
        if let Some(render_core_unsafe) = render_core_unsafe.as_mut() as Option<&mut render::RenderCoreUnsafe> {
            let render_core: &mut render::RenderCore = unsafe { render_core_unsafe.make_safe() };
            // Get camera transform.
            let camera_transform: CameraTransform = scene_data.camera_transform;

            // Iterate through components.
            for (mesh, node) in (&mut mesh_components, &nodes).join() {
                let transform: render::RenderTransform = render::RenderTransform::new(node.get_trans(), camera_transform.view, camera_transform.projection);
                mesh.render(transform, &mut mesh_pipeline, render_core);
            }
        }
    }
}