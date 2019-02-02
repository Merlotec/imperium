use crate::*;
use node::*;
use spatial::pipe::mesh::*;
use spatial::model::BufferedMesh;
use spatial::RenderComponent;
use scene::*;
use spatial::light::*;
use spatial::material::*;
use spatial::pass::SpatialPass;
use specs::prelude::*;

use std::time::Instant;
use std::time::Duration;

use specs_hierarchy::Hierarchy;

pub struct LightSystem;

impl <'a> System<'a> for LightSystem {
    type SystemData = (
        WriteExpect<'a, scene::GraphicsCapsule>,
        WriteExpect<'a, spatial::light::LightsController>,
        ReadStorage<'a, LightComponent>,
        ReadStorage<'a, node::NodeObject3D>,
    );

    fn run(&mut self, (mut graphics, mut lights_controller, lights, nodes): Self::SystemData) {
        // Here we update the shared lights buffer if we need to.
        let mut lights_list: LightsList = LightsList::new();
        for (light_component, node) in (&lights, &nodes).join() {
            let light_data: LightData = light_component.light.get_data(node.get_pos());
            lights_list.add_light(light_data);
        }

        lights_controller.lights = lights_list;
        if let Some(graphics) = unsafe { graphics.unsafe_borrow() } {
            lights_controller.update_buffer(&mut graphics.device);
        }
    }
}

pub struct MeshRenderSystem;

impl<'a> System<'a> for MeshRenderSystem {

    type SystemData = (
        WriteExpect<'a, scene::GraphicsCapsule>,
        WriteExpect<'a, SpatialPass>,
        ReadExpect<'a, SceneData>,
        WriteExpect<'a, MeshRenderPipeline>,
        ReadExpect<'a, spatial::light::LightsController>,
        WriteStorage<'a, BufferedMesh>,
        ReadStorage<'a, MaterialComponent>,
        ReadStorage<'a, node::NodeObject3D>,
    );

    fn run(&mut self, (mut graphics, mut render_pass, scene_data, mut mesh_pipeline, lights, mut meshes, materials, nodes): Self::SystemData) {
        // Only render if the render core is valid.
        if let Some(mut graphics) = unsafe { graphics.unsafe_borrow() } {
            // Get camera transform.
            let camera_transform: CameraTransform = scene_data.camera_transform;
            // Lights
            //mesh_pipeline.intrinsic_descriptor_interface.set.write_input(lights.buffer.as_ref(), 1, &graphics.graphics.device);

            if let Some((mut frame, pass)) = render_pass.next(graphics) {
                frame.begin_render(graphics, | dispatch | {
                    mesh_pipeline.bind_pipeline(&mut dispatch.command_buffer);
                    dispatch.begin_render_pass_inline(Color::black(), pass, |graphics, encoder| {
                        for (mut mesh, material, node) in (&mut meshes, &materials, &nodes).join() {
                            let transform: render::RenderTransform = render::RenderTransform::new(node.get_trans(), camera_transform.view, camera_transform.projection);
                            mesh.render(transform, &material.buffer.descriptor_set, &mut mesh_pipeline, encoder);
                        }
                    });
                });
            }
        }
    }
}

pub struct NodeHierarchySystem;

impl<'a> System<'a> for NodeHierarchySystem {

    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Hierarchy<scene::Parent>>,
        ReadStorage<'a, scene::Parent>,
        WriteStorage<'a, node::NodeObject3D>,
    );

    fn run(&mut self, (entities, hierarchy, parents, mut nodes): Self::SystemData) {
        for entity in hierarchy.all() {
            let mut offset: Matrix4f = Matrix4f::identity();
            if let Some(parent) = parents.get(*entity) {
                if let Some(node) = nodes.get(parent.parent_entity()) {
                    offset = node.get_trans();
                }
            }
            if let Some(node) = nodes.get_mut(*entity) {
                node.offset = offset;
            }
        }
    }

}