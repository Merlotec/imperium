use crate::*;
// Import specs here and expose through this module.
pub use specs::prelude::*;
pub use specs_hierarchy::Hierarchy;
pub use specs_hierarchy::HierarchySystem;
pub use specs_hierarchy::Parent as HierarchyParent;

use node::Node;

use std::mem;

pub struct Parent {
    entity: Entity,
}

impl Parent {
    pub fn new(entity: Entity) -> Self {
        return Self { entity };
    }
}

impl HierarchyParent for Parent {

    fn parent_entity(&self) -> Entity {
        return self.entity;
    }

}

impl specs::Component for Parent {

    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;

}

pub struct GraphicsCapsule {
    graphics: Option<*mut render::Graphics>,
}

impl GraphicsCapsule {

    pub fn new() -> Self {
        return Self { graphics: None };
    }

    pub fn lend_graphics(&mut self, graphics: *mut render::Graphics) {
        self.graphics = Some(graphics);
    }

    pub fn invalidate(&mut self) {
        self.graphics = None;
    }

    pub unsafe fn unsafe_borrow(&mut self) -> Option<&mut render::Graphics> {
        if let Some(graphics) = self.graphics {
            return Some(&mut *graphics);
        } else {
            return None;
        }
    }

}

unsafe impl Send for GraphicsCapsule {}
unsafe impl Sync for GraphicsCapsule {}

/// This marker trait should be implemented by an `Aggregator` to show that component `C` is implemented intrinsically by the aggregator.
pub trait HasIntrinsic<C: ComponentOf<Self>> : Aggregator where Self : Sized {}

/// A marker trait which denotes that a component can be used with a specific `Aggregator`.
pub trait ComponentOf<A: Aggregator + ?Sized> : specs::Component + Send + Sync {}

/// A node which holds the primary component `C` with the other necessary intrinsic components filled in.
/// This type should be redefined to include the `A` type parameter depending on the `Aggregator` used.
/// Custom implementations of this should be added where `A` is a specific `Aggregator`.
pub struct PrimaryEntity<A: Aggregator, C: ComponentOf<A>> {
    pub entity: Entity,
    phantom_a:  std::marker::PhantomData<A>,
    phantom_c:  std::marker::PhantomData<C>,
}

impl<A: Aggregator, C: ComponentOf<A>> PrimaryEntity<A, C> {
    pub fn new(entity: Entity) -> Self {
        return Self { entity, phantom_a: std::marker::PhantomData, phantom_c: std::marker::PhantomData };
    }
}

pub struct BaseEntity<A: Aggregator> {
    pub entity: Entity,
    phantom_a:  std::marker::PhantomData<A>,
}

impl<A: Aggregator> BaseEntity<A> {
    pub fn new(entity: Entity) -> Self {
        return Self { entity, phantom_a: std::marker::PhantomData, };
    }
}

pub trait Camera {
    fn camera_transform(&self, node: &Node) -> CameraTransform;
    fn absolute_transform(&self, node: &Node) -> Matrix4f {
        let camera_transform = self.camera_transform(node);
        return camera_transform.projection * camera_transform.view;
    }
}

#[derive(Copy, Clone)]
pub struct CameraTransform {
    pub projection: Matrix4f,
    pub view: Matrix4f,
}

impl CameraTransform {
    pub fn new(projection: Matrix4f, view: Matrix4f) -> Self {
        return Self { projection, view };
    }
    pub fn camera_matrix(&self) -> Matrix4f {
        return self.view.inverse_transform().log_expect("MATRIX ERROR");
    }
}

pub struct SceneData {
    pub camera_transform: CameraTransform,
}

impl SceneData {
    pub fn new() -> Self {
        return Self { camera_transform: CameraTransform::new(Matrix4f::identity(), Matrix4f::identity()) };
    }
}

pub trait Aggregator {

    type Camera: Camera + ComponentOf<Self> + Sized;

    type Node: Node + ComponentOf<Self>;

    /// Add default components to entity.
    fn build_entity(mut entity_builder: EntityBuilder) -> EntityBuilder where Self : Sized;

    /// Register resources and systems.
    fn load<'a, 'b : 'a>(&mut self, graphics: &mut render::Graphics, dispatcher_builder: scene::DispatcherBuilder<'a, 'b>, world: &mut scene::World) -> scene::DispatcherBuilder<'a, 'b>;

    /// Update resources.
    /// Systems are automatically run.
    fn dispatch_systems(&mut self, world: &mut World, dispatcher: &mut Dispatcher, graphics: &mut render::Graphics);

}

pub struct Scene<'a, 'b : 'a, A: Aggregator> {
    pub aggregator: A,
    pub world: World,
    pub dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b : 'a, A: Aggregator> Scene<'a, 'b, A>
    where <<A as scene::Aggregator>::Camera as specs::Component>::Storage: std::default::Default,
          <<A as scene::Aggregator>::Node as specs::Component>::Storage: std::default::Default {

    /// Creates a new scene with all the systems registered.
    pub fn create(mut aggregator: A, graphics: &mut render::Graphics) -> Self  {
        let mut world: World = World::new();
        Self::register_resources(&mut world);
        let mut dispatcher_builder = Self::register_systems(DispatcherBuilder::new());
        let mut dispatcher_builder = aggregator.load(graphics, dispatcher_builder, &mut world);
        let mut dispatcher: Dispatcher = dispatcher_builder.build();
        // Now we start call ths `on_start` method on the systems.
        dispatcher.setup(&mut world.res);
        return Self { aggregator, world, dispatcher };
    }

    fn register_resources(world: &mut World)  {
        world.add_resource(SceneData::new());
        world.add_resource::<Option<render::DispatchUnsafe>>(None);
        world.register::<A::Camera>();
        world.register::<A::Node>();
    }

    fn register_systems<'c, 'd : 'c>(dispatcher_builder: DispatcherBuilder<'c, 'd>) -> DispatcherBuilder<'c, 'd> {
        dispatcher_builder.with(HierarchySystem::<Parent>::new(), "hierarchy_system", &[])
    }

    pub fn create_primary_entity<C: ComponentOf<A>>(&mut self, component: C) -> PrimaryEntity<A, C> {
        let entity: Entity = A::build_entity(self.world.create_entity().with(component)).build();
        return PrimaryEntity::new(entity);
    }

    pub fn create_primary_entity_from<C: ComponentOf<A>>(&mut self, component: C, mut builder: EntityBuilder) -> PrimaryEntity<A, C> {
        let entity: Entity = A::build_entity(builder.with(component)).build();
        return PrimaryEntity::new(entity);
    }

    pub fn create_base_entity(&mut self) -> BaseEntity<A> {
        let entity: Entity = A::build_entity(self.world.create_entity()).build();
        return BaseEntity::new(entity);
    }

    pub fn basic_builder(&mut self) -> EntityBuilder {
        return A::build_entity(self.world.create_entity());
    }

    // Interior mutability on return type.
    pub fn get_scene_data(&self) -> specs::shred::FetchMut<SceneData> {
        return self.world.write_resource::<SceneData>();
    }

    pub fn update_scene_data(&self) {
        // Get updated camera data.
        let camera_fetch = self.world.read_storage::<A::Camera>();
        let node_fetch = self.world.read_storage::<A::Node>();
        // We use the last camera as the active one.
        for (node_component, camera_component) in (&node_fetch, &camera_fetch).join() {
            let camera_transform = camera_component.camera_transform(node_component);
            self.get_scene_data().camera_transform = camera_transform;
        }
    }

    /// Dispatches all the systems in the scene which will cause the scene to be updated and rendered.
    pub fn dispatch_systems(&mut self, graphics: &mut render::Graphics) {
        self.update_scene_data();
        self.aggregator.dispatch_systems(&mut self.world, &mut self.dispatcher, graphics);
    }

}