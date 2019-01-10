use crate::*;

use std::ops::Deref;
use std::ops::DerefMut;

use std::mem;

use node::Node3D;
use node::NodeObject3D;

use physics::nphysics3d::volumetric::Volumetric;

use gfx::Device;

pub mod model;
pub mod light;

pub mod pipe;
pub mod sys;

use specs::prelude::*;

use self::pipe::mesh::*;

pub type Scene3D<'a, 'b> = scene::Scene<'a, 'b, Spatial>;
pub type PrimaryEntity3D<C: scene::ComponentOf<Spatial>> = scene::PrimaryEntity<Spatial, C>;

/// The NodeObject3D can be used as a Spatial component.
impl scene::ComponentOf<Spatial> for NodeObject3D {}

/// The spatial aggregator for use with a `Scene`.
pub struct Spatial;

impl scene::HasIntrinsic<NodeObject3D> for Spatial {}

impl scene::Aggregator for Spatial {

    /// Not `Self` - that would be cyclic.
    /// Here `self` (lower case `s`) denotes this module.
    type Camera = self::Camera;

    fn build_entity(mut entity_builder: scene::EntityBuilder) -> scene::Entity where Self : Sized {
        return entity_builder.with(NodeObject3D::new()).build();
    }
    fn load<'a, 'b : 'a>(&mut self, renderer: &mut render::Renderer, dispatcher_builder: scene::DispatcherBuilder<'a, 'b>, world: &mut scene::World) -> scene::DispatcherBuilder<'a, 'b> {
        world.register::<NodeObject3D>();
        world.register::<model::MeshComponent>();

        world.add_resource(MeshRenderPipeline::create(&mut renderer.graphics.device, &mut renderer.graphics.render_pass));

        dispatcher_builder.with(sys::MeshRenderSystem, "render::mesh", &[])
    }
    fn update(&mut self, world: &mut scene::World) {

    }
}

impl<A: scene::Aggregator, C: scene::ComponentOf<A>> scene::PrimaryEntity<A, C>
    where NodeObject3D : scene::ComponentOf<A>, A : scene::HasIntrinsic<NodeObject3D>  {
    pub fn get_node<'a, 'b>(&'a self, world: &'b scene::World) -> Option<&'b node::Node3D> {
        let node_storage = world.read_storage::<node::NodeObject3D>();
        if let Some(cmp) = node_storage.get(self.entity) {
            return Some(unsafe { mem::transmute(cmp as &node::Node3D) });
        }
        return None;
    }

    /// I think that this is allowed because the storage references data in the World object.
    /// This means that even if the storage object is destroyed, the data it points to is still valid.s
    pub fn node_mut<'a, 'b>(&'a self, world: &'b mut scene::World) -> Option<&'b mut node::Node3D> {
        let mut node_storage = world.write_storage::<node::NodeObject3D>();
        if let Some(cmp) =  node_storage.get_mut(self.entity) {
            return Some(unsafe { mem::transmute(cmp as &mut node::Node3D) });
        }
        return None;
    }
}

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
    pub fn render(&mut self, node: &mut ComponentNode) {
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

pub trait RenderComponent {

    type RenderPipeline;

    fn render(&mut self, transform: render::RenderTransform, pipeline: &mut Self::RenderPipeline, render_core: &mut render::RenderCore);

}

pub trait Component {

    fn update(&mut self, cycle: &mut UpdateCycle) {}

    fn render(&mut self, transform: Matrix4f, cycle: &mut RenderCycle);

    /// Should return the geometry of this component.
    /// There can be multiple geometry 'sections' for any physics component.
    fn create_physics(&self, physics_world: &mut physics::World) -> PhysicsData {
        return PhysicsData::none();
    }

    /// Updates the physics bodies of a component which manages physics.
    fn update_physics(&mut self, physics_data: &mut PhysicsData, physics_world: &mut physics::World) {}

}

/// Defines a component which can be rendered in batch by the scene.
/// The 'render_batch' function should call the necessary functions on the scene object in order to properly execute a batch render.
pub trait BatchRenderComponent {
    fn render_batch(&mut self, transforms: &[Matrix4f], cycle: &mut RenderCycle);
}

/// Defines a component which knows its own transform.
/// This will be the case for nodes, since they have their own transforms defined by their position, rotation and scale.
/// However, this trait can be implemented by things other than nodes, e.g. a World.
pub trait ComponentNode {
    fn update(&mut self, cycle: &mut UpdateCycle);
    fn render(&mut self, cycle: &mut RenderCycle);
    fn update_physics(&mut self, physics_world: &mut physics::World);
}

/// The container struct for all the physics data of a node.
/// This includes bodies and will also include joints in the future.
pub struct PhysicsData {

    pub bodies: Vec<PhysicsBody>,

}

impl PhysicsData {
    pub fn none() -> Self {
        let bodies: Vec<PhysicsBody> = Vec::new();
        return Self { bodies };
    }
}

/// Contains geometric data for physics simulations.
pub struct PhysicsGeometry {

    /// The actual geometry.
    pub shape: physics::ShapeHandle,

    /// The position, in local coordinates, of this geometry.
    /// This can also be described as it's offset from the node's position.
    pub offset: Vector3f,

}

impl PhysicsGeometry {

    /// Creates a new physics geometry object with a cube shape.
    /// The length of each side will be x.
    pub fn cube(x: f32) -> PhysicsGeometry {
        return PhysicsGeometry::cuboid(Vector3f::new(x, x, x));
    }

    /// Creates a new cuboid physics shape that can be used for collision detection.
    /// The dimensions of the cuboid are specified by the `dimensions` parameter.
    pub fn cuboid(dimensions: Vector3f) -> PhysicsGeometry {
        let shape: physics::ShapeHandle = physics::ShapeHandle::new(
            physics::collide::shape::Cuboid::new(
                physics::na::Vector3::new(dimensions.x / 2.0, dimensions.y / 2.0, dimensions.z / 2.0)
            )
        );
        return PhysicsGeometry { shape, offset: Vector3f::zero() };
    }

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

    /// This gets the position of the central position from which this physics body is offset from.
    pub fn get_absolute_pos(&self, physics_world: &physics::World) -> Option<Vector3f> {
        if let Some(pos) = self.get_pos(physics_world) {
            return Some(pos - self.offset);
        }
        return None;
    }

    /// This gets the position of the central position from which this physics body is offset from.
    pub fn set_absolute_pos(&self, pos: Vector3f, physics_world: &mut physics::World) {
        self.set_pos(pos + self.offset, physics_world);
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

pub struct Node<T: Component> {
    pub node: node::NodeObject3D,
    pub physics_data: PhysicsData,
    pub component: T,
}

impl<T: Component> Node<T> {

    /// Creates a new node containing the specified instantiated component.
    pub fn new(component: T, physics_world: &mut physics::World) -> Self {
        let node: NodeObject3D = node::NodeObject3D::new();
        let physics_data: PhysicsData = component.create_physics(physics_world);
        return Self { node, physics_data, component };
    }

}

impl<T: Component> ComponentNode for Node<T> {
    fn update(&mut self, cycle: &mut UpdateCycle) {
        self.component.update(cycle);
    }
    fn render(&mut self, cycle: &mut RenderCycle) {
        self.component.render(self.node.get_trans(), cycle);
    }
    fn update_physics(&mut self, physics_world: &mut physics::World) {
        self.component.update_physics(&mut self.physics_data, physics_world);
    }
}

impl<T: Component> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &T {
        return &self.component;
    }
}

impl<T: Component> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut T {
        return &mut self.component;
    }
}

impl<T: Component> node::NodeImplementor3D for Node<T> {

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

impl scene::Camera for Camera {

    fn camera_transform(&self) -> scene::CameraTransform {
        return scene::CameraTransform::new(self.get_projection_matrix(), self.get_view_matrix());
    }

}

impl scene::Component for Camera {
    type Storage = scene::VecStorage<Self>;
}

impl scene::ComponentOf<Spatial> for Camera {}

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
    /// Updates the physics node.
    pub fn update_physics(&mut self, physics_node: &mut ComponentNode) {
        physics_node.update_physics(&mut self.physics_world);
    }

}

