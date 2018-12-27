use crate::*;

use gfx::Device;

pub mod model;
pub mod light;

pub mod mesh_pipeline;

use self::mesh_pipeline::*;

/// The RenderCycle object is an encapsulating structure which includes all data needed for rendering a scene.
/// This simply makes code more readable as we only need one parameter rather than three.
pub struct RenderCycle<'a, 'b : 'a> {

    pub scene: &'a mut Scene,
    pub graphics: &'a mut render::Graphics,
    pub encoder: &'a mut command::Encoder<'b>,

}

impl<'a, 'b : 'a> RenderCycle<'a, 'b> {

    /// Creates a new RenderCycle object with the specified mutable references.
    pub fn new(scene: &'a mut Scene, graphics: &'a mut render::Graphics, encoder: &'a mut command::Encoder<'b>) -> Self {
        return Self { scene, graphics, encoder };
    }

    /// Renders the node by calling its render function.
    pub fn render(&mut self, node: &mut RenderNode) {
        node.render(self);
    }

}

/// The UpdateCycle is similar to the RenderCycle object.
/// It contains all the data needed to update a Scene component.
pub struct UpdateCycle<'a> {

    pub scene: &'a mut Scene,
    pub renderer: &'a mut render::Renderer,
    pub delta: f32,

}

impl<'a> UpdateCycle<'a> {

    /// Creates a new UpdateCycle object with the specified mutable references.
    pub fn new(scene: &'a mut Scene, renderer: &'a mut render::Renderer, delta: f32) -> Self {
        return Self { scene, renderer, delta };
    }

}

/// Defines a component which can be rendered by the scene.
/// This only contains one function 'render' which should call the necessary functions on the scene in order for the component to render itself.
pub trait RenderComponent {
    fn render(&mut self, transform: Matrix4f, cycle: &mut RenderCycle);
}

/// Defines a component which can be rendered in batch by the scene.
/// The 'render_batch' function should call the necessary functions on the scene object in order to properly execute a batch render.
pub trait BatchRenderComponent {
    fn render_batch(&mut self, transforms: &[Matrix4f], cycle: &mut RenderCycle);
}

/// Defines a component which knows its own transform.
/// This will be the case for nodes, since they have their own transforms defined by their position, rotation and scale.
/// However, this trait can be implemented by things other than nodes, e.g. a World.
pub trait RenderNode {
    fn render(&mut self, cycle: &mut RenderCycle);
}

impl<T> RenderNode for node::ContainerNode3D<T> where T: RenderComponent {
    fn render(&mut self, cycle: &mut RenderCycle) {
        self.component.render(self.node.get_trans(), cycle);
    }
}

const CAMERA_FOV: f32 = 0.8;
const CAMERA_NEAR: f32 = 100.0;
const CAMERA_FAR: f32 = 1000000000000000000.0;

/// The camera structure contains transformation data which will transform the vertices in the world to represent the camera's view and projection.
pub struct Camera {

    pub fov: f32,
    pub aspect: f32,
    pub frame_size: Vector2f,

    pub projection: Matrix4f,
    pub node: node::NodeObject3D,

}

impl Camera {

    /// Creates a new camera with the specified viewport size.
    pub fn create(frame_size: Vector2f, fov: f32) -> Camera {
        let aspect: f32 = frame_size.x / frame_size.y;
        return Camera {
            fov: fov,
            aspect,
            frame_size,
            projection: Self::perspective_projection(aspect, fov, CAMERA_NEAR, CAMERA_FAR),
            //projection: cgmath::ortho(0.0, size.x, size.y, 0.0, -1000.0, 1000.0),
            node: node::NodeObject3D::new(),
        }
    }

    pub fn reframe(&mut self, frame_size: Vector2f) {
        let aspect: f32 = frame_size.x / frame_size.y;
        self.projection = Self::perspective_projection(aspect, self.fov, CAMERA_NEAR, CAMERA_FAR);
        self.frame_size = frame_size;
        self.aspect = aspect;
    }

    fn perspective_projection(aspect_ratio: f32, field_of_view: f32, near_plane: f32, far_plane: f32) -> Matrix4f {

        let f: f32 = 1.0 / (0.5 * field_of_view).tan();

        let matrix: Matrix4f = Matrix4f::new(
        f / aspect_ratio,
        0.0,
        0.0,
        0.0,

        0.0,
        -f,
        0.0,
        0.0,

        0.0,
        0.0,
        far_plane / (near_plane - far_plane),
        -1.0,

        0.0,
        0.0,
        (near_plane * far_plane) / (near_plane - far_plane),
        0.0
        );

        return matrix;
    }

    pub fn get_projection_matrix(&self) -> Matrix4f {
        return self.projection;
    }

    /// This matrix transforms the vertices which are in world space to camera space.
    /// This imitates the camera's movement, position and rotation.
    pub fn get_view_matrix(&self) -> Matrix4f {
        return self.node.get_trans().inverse_transform().expect("MATRIX ERROR!");
    }

    pub fn get_camera_matrix(&self) -> Matrix4f {
        return self.node.get_trans();
    }

    pub fn screen_to_world(&self, pos: Vector2f) -> Vector3f {
        let ray_nds: Vector3f = Vector3f::new(
            (pos.x - (self.frame_size.x / 2.0)) / (self.frame_size.x / 2.0),
            (pos.y - (self.frame_size.y / 2.0)) / (self.frame_size.y / 2.0),
            0.0
        );

        let ray_clip: Vector4f = Vector4f::new(
            ray_nds.x,
            ray_nds.y,
            1.0,
            1.0,
        );

        let ray_eye: Vector4f = self.get_projection_matrix().invert().unwrap() * ray_clip;
        let ray_world: Vector4f = self.get_camera_matrix() * ray_eye;

        return Vector3f::new(ray_world.x, ray_world.y, ray_world.z).normalize();

    }

}

impl node::NodeImplementor3D for Camera {

    fn get_node_obj(&self) -> &node::NodeObject3D {
        return &self.node;
    }

    fn get_node_obj_mut(&mut self) -> &mut node::NodeObject3D {
        return &mut self.node;
    }

}

/// The Scene struct contains data necessary for rendering a scene.
/// NOTE: components are not contained within the scene, and neither are lights.
/// However the raw light data that is sent to the shader is stored so that lights do not have to be re-uploaded on every frame.
/// This increases efficiency.
pub struct Scene {

    /// The camera can be used to transform the scene components so that they render as if they were being viewed by a camera.
    pub camera: Camera,

    /// The lights list is the raw shader data which is uploaded to the GPU whenever 'upload_lights' is called.
    /// This should only be called when the lights are changed.
    pub light_list: LightList,

    pub mesh_pipeline: MeshRenderPipeline,

    /// A physics world that can be used for physics simulation.
    pub physics_world: physics::World,

}

impl Scene {

    /// Creates a new emtpy scene with a camera pointing forward with perspective projection.
    /// There are no lights in the scene, and all lights should be added and uploaded manually using 'apply_light' then 'upload_lights' when all the lights have been added.
    pub fn create(graphics: &mut render::Graphics) -> Scene {
        let camera = Camera::create(graphics.render_surface.get_size(), 0.9);
        let mesh_pipeline: MeshRenderPipeline = MeshRenderPipeline::create(&mut graphics.device, &graphics.render_pass);
        let light_list: LightList = LightList::new();
        return Scene { camera, light_list, mesh_pipeline };
    }

    /// Removes all lights from the light list by creating a new, empty light list.
    /// This can be used if lights are added each frame before rendering.
    /// The use of this function depends on the rendering technique used.
    pub fn clear_lights(&mut self) {
        self.light_list = LightList::new();
    }

    pub fn apply_light(&mut self, light: &light::Light) {
        self.add_light_data(light.get_data());
    }

    pub fn add_light_data(&mut self, light_data: LightData) {
        self.light_list.add_light(light_data);
    }

    /// Begins the render cycle by creating the render cycle object for the scene.
    pub fn begin_render_cycle<'a, 'b : 'a>(&'a mut self, graphics: &'a mut render::Graphics, encoder: &'a mut command::Encoder<'b>) -> RenderCycle<'a, 'b> {
        self.mesh_pipeline.reset();
        return RenderCycle::new(self, graphics, encoder);
    }

    /// If lighting has changed since the last upload, this should be called before any rendering takes place to upload the lighting to the gpu.
    /// If no changes have been made to the light_list object since it was last uploaded, there is no need to call this function.
    /// Once this function is called any changes made to the lights in the scene will not be reflected in the render until this function is called again.
    pub fn upload_lights(&self, device: &core::Device) {
        self.mesh_pipeline.upload_lights(self.light_list, device);
    }

    /// Updates the scene to be ready to render another frame.
    pub fn begin_update_cycle<'a>(&'a mut self, renderer: &'a mut render::Renderer, delta: f32) -> UpdateCycle<'a> {
        return UpdateCycle::new(self, renderer, delta);
    }

    pub fn render_mesh(&mut self, vertex_input: &pipeline::VertexInput, texture_set: &pipeline::DescriptorSet, transform: render::RenderTransform, graphics: &mut render::Graphics, encoder: &mut command::Encoder) {
        self.mesh_pipeline.render(vertex_input, texture_set, transform, graphics, encoder);
    }

    pub fn render_mesh_batch(&mut self, vertex_input: &pipeline::VertexInput, texture_set: &pipeline::DescriptorSet, transforms: &[render::RenderTransform], graphics: &mut render::Graphics, encoder: &mut command::Encoder) {
        self.mesh_pipeline.render_batch(vertex_input, texture_set, transforms, graphics, encoder);
    }

}

