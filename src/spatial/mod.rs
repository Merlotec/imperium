use crate::*;

use node::Node3D;
use node::NodeObject3D;

use physics::nphysics3d::volumetric::Volumetric;

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

/// This trait defines standard update behaviour for a scene node.
pub trait SceneComponent {

    fn update(&mut self, node: &mut Node3D, scene: &mut Scene);

}

pub trait SceneNode {

    fn update(&mut self, scene: &mut Scene);

}

pub struct PhysicsGeometry {

    /// The actual geometry.
    pub shape: physics::ShapeHandle,

    /// The position, in local coordinates, of this geometry.
    /// This can also be described as it's offset from the node's position.
    pub offset: Vector3f,

}

const COLLIDER_MARGIN: f32 = 0.01;

/// Defines physics data for a node.
/// This struct holds a handle to the actual physics body held in a physics world.
/// Therefore, the physics world is needed for most operations.
pub struct PhysicsBody {

    /// This handle references the actual physics body data in a physics world.
    pub handle: physics::BodyHandle,

    /// This offset vector represents the offset of the rigid body coordinates from the world coordinates of the actual node.
    /// This may be needed if the physics body does not represent the actual component.
    pub offset: Vector3f,

}

impl PhysicsBody {

    /// Creates a new physics body which can be used in collision detection.
    pub fn create(geometry: PhysicsGeometry, physics_world: &mut physics::World) -> Self {
        let inertia = geometry.shape.inertia(1.0);
        let center_of_mass = geometry.shape.center_of_mass();
        let pos: physics::na::Isometry<f32, _, _> = physics::na::Isometry3::new(physics::na::Vector3::new(geometry.offset.x, geometry.offset.y, geometry.offset.z), physics::na::zero());
        let handle = physics_world.add_rigid_body(pos, inertia, center_of_mass);
        physics_world.add_collider(
            COLLIDER_MARGIN,
            geometry.shape.clone(),
            handle,
            physics::na::Isometry3::identity(),
            physics::physical::object::Material::default(),
        );
        if let Some(body) = physics_world.rigid_body_mut(handle) {
            body.set_status(physics::physical::object::BodyStatus::Static);

        }
        physics_world.activate_body(handle);
        return Self { handle, offset: geometry.offset };
    }

    /// Gets the position of the physics body in the specified world.
    pub fn get_pos(&self, physics_world: &physics::World) -> Option<Vector3f> {
        if let Some(body) = self.get_body(physics_world) {
            let vec = body.position().translation.vector;
            return Some(Vector3f::new(vec.x, vec.y, vec.z));
        }
        return None;
    }

    /// Sets the position of the physics body in the specified world.
    pub fn set_pos(&self, pos: Vector3f, physics_world: &mut physics::World) {
        if let Some(body) = self.get_body_mut(physics_world) {
            let mut position: physics::na::Isometry<f32, _, _> = physics::na::Isometry3::new(physics::na::Vector3::new(pos.x, pos.y, pos.z), physics::na::zero());
            position.translation.vector = physics::na::Vector3::new(pos.x, pos.y, pos.z);
            body.set_position(position);
        }
    }

    /// Removes this physics body from the specified world.
    pub fn remove_from_world(&self, physics_world: &mut physics::World) {
        physics_world.remove_bodies(&[self.handle]);
    }

    /// Gets an immutable reference to a body.
    pub fn get_body<'a>(&self, physics_world: &'a physics::World) -> Option<&'a physics::RigidBody> {
        return physics_world.rigid_body(self.handle);
    }

    /// Gets a mutable reference to a body.
    pub fn get_body_mut<'a>(&self, physics_world: &'a mut physics::World) -> Option<&'a mut physics::RigidBody> {
        return physics_world.rigid_body_mut(self.handle);
    }

}

/// Represents a physics node which contains its own physics data.
pub trait PhysicsNode {
    fn update_physics(&mut self, physics_world: &mut physics::World);
}

/// This trait defines a component which should respond to physics.
pub trait PhysicsComponent {

    /// Should return the geometry of this component.
    /// There can be multiple geometry 'sections' for any physics component.
    fn get_physics_geometry(&self) -> Vec<PhysicsGeometry>;

    /// Updates the physics bodies of a component which manages physics.
    fn update_physics(&mut self, bodies: &mut Vec<PhysicsBody>, physics_world: &mut physics::World);

}

pub struct Node<T> {
    pub node: node::NodeObject3D,
    pub physics_bodies: Vec<PhysicsBody>,
    pub component: T,
}

impl<T> Node<T> {

    /// Creates a new node containing the specified instantiated component.
    pub fn new(component: T) -> Self {
        let node = node::NodeObject3D::new();
        let physics_bodies = Vec::new();
        return Self { node, physics_bodies, component };
    }

}

impl<T> RenderNode for Node<T> where T : RenderComponent {
    fn render(&mut self, cycle: &mut RenderCycle) {
        self.component.render(self.node.get_trans(), cycle);
    }
}

impl<T> PhysicsNode for Node<T> where T : PhysicsComponent {
    fn update_physics(&mut self, physics_world: &mut physics::World) {
        //TODO: We need to be able to represent rotation as well as translation.
        if let Some(body) = self.physics_bodies.first() {
            if let Some(pos) = body.get_pos(physics_world) {
                self.node.set_pos(pos);
            }
        }
        self.component.update_physics(&mut self.physics_bodies, physics_world);
    }
}

impl<T> SceneNode for Node<T> where T : SceneComponent {
    fn update(&mut self, scene: &mut Scene) {
        self.component.update(&mut self.node, scene);
    }
}

impl<T> node::NodeImplementor3D for Node<T> {

    fn get_node_obj(&self) -> &NodeObject3D {
        return &self.node;
    }

    fn get_node_obj_mut(&mut self) -> &mut NodeObject3D {
        return &mut self.node;
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

    pub fn set_frame_size(&mut self, frame_size: Vector2f) {
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
        let physics_world: physics::World = physics::World::new();
        return Scene { camera, light_list, mesh_pipeline, physics_world };
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

    /// Updates a standard scene node.
    pub fn update_node(&mut self, node: &mut SceneNode) {
        node.update(self);
    }

    /// Updates the physics node.
    pub fn update_physics(&mut self, physics_node: &mut PhysicsNode) {
        physics_node.update_physics(&mut self.physics_world);
    }

}

