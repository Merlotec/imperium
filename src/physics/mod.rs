use crate::*;
pub extern crate nphysics3d;
pub extern crate ncollide3d;
pub extern crate nalgebra;
pub use self::nphysics3d as physical;
pub use self::ncollide3d as collide;
pub use self::nalgebra as na;

pub use nphysics3d::object::BodyHandle;
pub use ncollide3d::world::CollisionGroups;

pub type World = nphysics3d::world::World<f32>;
pub type Ray = ncollide3d::query::Ray<f32>;
pub type ShapeHandle = ncollide3d::shape::ShapeHandle<f32>;
pub type Body<'a> = nphysics3d::object::Body<'a, f32>;
pub type RigidBody = nphysics3d::object::RigidBody<f32>;