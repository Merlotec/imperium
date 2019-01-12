use crate::*;
use node::*;
use spatial::pipe::mesh::*;
use spatial::MeshComponent;
use spatial::RenderComponent;
use scene::*;
use spatial::light::*;
use spatial::material::*;

pub struct LightSystem;

impl <'a> System<'a> for LightSystem {
    type SystemData = (
        WriteExpect<'a, Option<render::RenderCoreUnsafe>>,
        WriteExpect<'a, spatial::light::LightsController>,
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, node::NodeObject3D>,
    );

    fn run(&mut self, (mut render_core_unsafe, mut lights_controller, lights, nodes): Self::SystemData) {
        // Here we update the shared lights buffer if we need to.
        let mut lights_list: LightsList = LightsList::new();
        for (light_component, node) in (&lights, &nodes).join() {
            let light_data: LightData = light_component.light.get_data(node.get_pos());
            lights_list.add_light(light_data);
        }

        lights_controller.lights = lights_list;
        if let Some(render_core_unsafe) = render_core_unsafe.as_mut() as Option<&mut render::RenderCoreUnsafe> {
            let render_core: &mut render::RenderCore = unsafe { render_core_unsafe.make_safe() };
            lights_controller.update_buffer(&mut render_core.graphics.device);
        }
    }
}

pub struct MeshRenderSystem;

impl<'a> System<'a> for MeshRenderSystem {

    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Option<render::RenderCoreUnsafe>>,
        ReadExpect<'a, SceneData>,
        WriteExpect<'a, MeshRenderPipeline>,
        ReadExpect<'a, spatial::light::LightsController>,
        WriteStorage<'a, MeshComponent>,
        ReadStorage<'a, node::NodeObject3D>,
        ReadStorage<'a, MaterialComponent>
    );

    fn run(&mut self, (entities, mut render_core_unsafe, scene_data, mut mesh_pipeline, lights, mut mesh_components, nodes, material_components): Self::SystemData) {
        // Only render if the render core is valid.
        if let Some(render_core_unsafe) = render_core_unsafe.as_mut() as Option<&mut render::RenderCoreUnsafe> {
            let render_core: &mut render::RenderCore = unsafe { render_core_unsafe.make_safe() };
            // Get camera transform.
            let camera_transform: CameraTransform = scene_data.camera_transform;

            // Lights
            mesh_pipeline.intrinsic_descriptor_interface.set.write_input(lights.buffer.as_ref(), 1, &render_core.graphics.device);

            // Iterate through components.
            for (entity, mesh, node) in (&entities, &mut mesh_components, &nodes).join() {

                for mesh_comp in mesh.meshes.iter_mut() {
                    if let Some(material) = material_components.get(entity) {
                        mesh_comp.set_material_input(material.buffer.as_ref(), &render_core.graphics.device);
                    }

                    let transform: render::RenderTransform = render::RenderTransform::new(node.get_trans(), camera_transform.view, camera_transform.projection);
                    mesh_comp.render(transform, &mut mesh_pipeline, render_core);
                }
            }
        }
    }
}