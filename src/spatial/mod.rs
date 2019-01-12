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
pub mod material;

pub mod pipe;
pub mod sys;

use specs::prelude::*;
use self::light::*;
use node::*;

use self::pipe::mesh::*;

pub type Scene3D<'a, 'b> = scene::Scene<'a, 'b, Spatial>;

impl<'a, 'b> Scene3D<'a, 'b> {
    pub fn create_3d(renderer: &mut render::Renderer) -> Self {
        return Self::create(Spatial, renderer);
    }
}

pub type PrimaryEntity3D<C: scene::ComponentOf<Spatial>> = scene::PrimaryEntity<Spatial, C>;

/// The NodeObject3D can be used as a Spatial component.
impl scene::ComponentOf<Spatial> for NodeObject3D {}
impl scene::ComponentOf<Spatial> for LightComponent {}
impl scene::ComponentOf<Spatial> for MeshComponent {}

/// The spatial aggregator for use with a `Scene`.
pub struct Spatial;

impl scene::HasIntrinsic<NodeObject3D> for Spatial {}

impl scene::Aggregator for Spatial {

    /// Not `Self` - that would be cyclic.
    /// Here `self` (lower case `s`) denotes this module.
    type Camera = self::Camera;

    type Node = node::NodeObject3D;

    fn build_entity(mut entity_builder: scene::EntityBuilder) -> scene::Entity where Self : Sized {
        return entity_builder.with(NodeObject3D::new()).build();
    }
    fn load<'a, 'b : 'a>(&mut self, renderer: &mut render::Renderer, dispatcher_builder: scene::DispatcherBuilder<'a, 'b>, world: &mut scene::World) -> scene::DispatcherBuilder<'a, 'b> {
        // Camera and node types already registered.
        // Here we register additional types.
        world.register::<MeshComponent>();
        world.register::<light::LightComponent>();
        world.register::<material::MaterialComponent>();

        world.add_resource(MeshRenderPipeline::create(&mut renderer.graphics.device, &mut renderer.graphics.render_pass));
        world.add_resource(LightsController::new(&mut renderer.graphics.device));

        dispatcher_builder.with(sys::MeshRenderSystem, "render::mesh", &[]).with(sys::LightSystem, "helper::light", &[])
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

pub trait RenderComponent {

    type RenderPipeline;

    fn render(&mut self, transform: render::RenderTransform, pipeline: &mut Self::RenderPipeline, render_core: &mut render::RenderCore);

}

/// Defines a component which can be rendered in batch by the scene.
/// The 'render_batch' function should call the necessary functions on the scene object in order to properly execute a batch render.
/*
pub trait BatchRenderComponent {
    fn render_batch(&mut self, transforms: &[Matrix4f], cycle: &mut RenderCycle);
}
*/

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

impl scene::Camera for Camera {

    fn camera_transform(&self, node: &node::Node) -> scene::CameraTransform {
        return scene::CameraTransform::new(self.get_projection_matrix(), node.get_trans().inverse_transform().expect("UNEXPECTED MATRIX ERROR"));
    }

}

impl scene::Component for Camera {
    type Storage = scene::VecStorage<Self>;
}

impl scene::ComponentOf<Spatial> for Camera {}

pub struct MeshComponent {

    pub meshes: Vec<model::BufferedMesh>,

}

impl MeshComponent {

    pub fn new(meshes: Vec<model::BufferedMesh>) -> Self {
        return Self { meshes };
    }

    pub fn from_model(model: &model::Model, textures: Vec<&texture::Texture>, pipeline: &mut MeshRenderPipeline, renderer: &mut render::Renderer) -> Self {
        let mut meshes: Vec<model::BufferedMesh> = Vec::with_capacity(model.meshes.len());

        let mut i: usize = 0;
        for mesh in model.meshes.iter() {
            if let Some(tex) = textures.get(i) {
                let mesh: model::BufferedMesh = model::BufferedMesh::new(&mesh, tex, pipeline, renderer);
                meshes.push(mesh);
            }
            i += 1;
        }
        return Self::new(meshes);
    }

    pub fn render_meshes(&mut self, transform: render::RenderTransform, pipeline: &mut MeshRenderPipeline, render_core: &mut render::RenderCore) {
        for mesh in self.meshes.iter_mut() {
            mesh.render(transform, pipeline, render_core)
        }
    }

}

impl specs::Component for MeshComponent {
    type Storage = specs::VecStorage<Self>;
}

pub struct MetaRenderComponent {



}