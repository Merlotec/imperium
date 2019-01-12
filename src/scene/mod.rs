use crate::*;
// Import specs here and expose through this module.
pub use specs::prelude::*;

use node::Node;

use std::mem;

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

pub trait Camera {
    fn camera_transform(&self, node: &Node) -> CameraTransform;
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
    fn build_entity(mut entity_builder: EntityBuilder) -> Entity where Self : Sized;

    /// Register resources and systems.
    fn load<'a, 'b : 'a>(&mut self, renderer: &mut render::Renderer, dispatcher_builder: scene::DispatcherBuilder<'a, 'b>, world: &mut scene::World) -> scene::DispatcherBuilder<'a, 'b>;

    /// Update resources.
    /// Systems are automatically run.
    fn update(&mut self, world: &mut World);

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
    pub fn create(mut aggregator: A, renderer: &mut render::Renderer) -> Self  {
        let mut world: World = World::new();
        Self::register_resources(&mut world);
        let mut dispatcher_builder = DispatcherBuilder::new();
        let mut dispatcher_builder = aggregator.load(renderer, dispatcher_builder, &mut world);
        let mut dispatcher: Dispatcher = dispatcher_builder.build();
        // Now we start call ths `on_start` method on the systems.
        dispatcher.setup(&mut world.res);
        return Self { aggregator, world, dispatcher };
    }

    fn register_resources(world: &mut World)  {
        world.add_resource(SceneData::new());
        world.add_resource::<Option<render::RenderCoreUnsafe>>(None);
        world.register::<A::Camera>();
        world.register::<A::Node>();
    }

    pub fn create_primary_entity<C: ComponentOf<A>>(&mut self, component: C) -> PrimaryEntity<A, C> {
        let entity: Entity = A::build_entity(self.world.create_entity().with(component));
        return PrimaryEntity::new(entity);
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
    pub fn dispatch_systems(&mut self, graphics: &mut render::Graphics, encoder: &mut command::Encoder) {
        self.update_scene_data();
        self.aggregator.update(&mut self.world);
        let render_core_unsafe: render::RenderCoreUnsafe = render::RenderCoreUnsafe::new(graphics, encoder);
        // We may need some unsafe magic... THE THRILL
        // Its ok though because the object will not live beyond this function - we invalidate it before the pointer 'borrow' is over.
        let render_resource: Option<render::RenderCoreUnsafe> = unsafe { Some(render_core_unsafe) };
        self.world.add_resource::<Option<render::RenderCoreUnsafe>>(render_resource);
        // Dispatch.
        self.dispatcher.dispatch(&self.world.res);
        // Invalidate renderer to ensure no unsafe uses.
        let mut render_resource_live = self.world.write_resource::<Option<render::RenderCoreUnsafe>>();
        render_resource_live.take();
    }

}