use crate::*;

use std::ops::Deref;
use std::ops::DerefMut;

pub const UNBOUND_FIXED_SIZE: Vector2f = Vector2f { x: 10.0, y: 10.0 };

#[derive(Copy, Clone)]
/**
The default node object contains the data necessary to handle a basic node (position and transform).
*/
pub struct NodeObject2D {

    pub trans: Matrix4f,
    pub offset: Matrix4f,
    pub rotation: Vector2f,

}

impl NodeObject2D {

    pub fn new() -> NodeObject2D {

        return NodeObject2D { trans: Matrix4f::identity(), offset: Matrix4f::identity(), rotation: Vector2f::new(0.0, 0.0) };

    }

    pub fn from(trans: Matrix4f) -> NodeObject2D {

        return NodeObject2D { trans: Matrix4f::identity(), offset: Matrix4f::identity(), rotation: Vector2f::new(0.0, 0.0) };

    }

    pub fn from_pos_and_scale(pos: Vector2f, scale: Vector2f) -> NodeObject2D {

        let mut trans: Matrix4f = Matrix4f::identity();
        trans.set_translation(pos.to_vec3());
        trans.set_scale(scale.to_vec3());

        return NodeObject2D { trans: Matrix4f::identity(), offset: Matrix4f::identity(), rotation: Vector2f::new(0.0, 0.0) };

    }

}

impl NodeObject2D {

    pub fn set_pos(&mut self, pos: Vector2f) {
        self.trans.set_translation(pos.to_vec3());
    }
    pub fn get_pos(&self) -> Vector2f {
        return self.trans.get_translation().to_vec2();
    }
    pub fn set_scale(&mut self, scale: Vector2f) {
        self.trans.set_scale(scale.to_vec3());
    }
    pub fn get_scale(&self) -> Vector2f {
        return self.trans.get_scale().to_vec2();
    }
    pub fn set_rotation(&mut self, rotation: Vector2f) {
        self.rotation = rotation;
    }
    pub fn get_rotation(&self) -> Vector2f {
        return self.rotation;
    }
    pub fn set_trans(&mut self, trans: Matrix4f) {
        self.trans = trans;
    }
    pub fn get_trans(&self) -> Matrix4f {
        let rx = Matrix4f::from_angle_x(cgmath::Rad(self.rotation.x));
        let ry = Matrix4f::from_angle_z(cgmath::Rad(self.rotation.y));

        let rotation: Matrix4f = rx * ry;
        return self.offset * self.trans * rotation;
    }
    pub fn set_offset(&mut self, offset: Matrix4f) {

        // The inherited offset can only be a translation otherwise there will be issues with size and scaling.
        self.offset = Matrix4f::from_translation(offset.get_translation());

    }
    pub fn get_offset(&self) -> Matrix4f {
        return self.offset;
    }

}

impl SizedNodeImplementor2D for NodeObject2D {}

pub trait Node2D {

    fn set_pos(&mut self, pos: Vector2f);
    fn get_pos(&self) -> Vector2f;
    fn set_scale(&mut self, scale: Vector2f);
    fn get_scale(&self) -> Vector2f;
    fn set_rotation(&mut self, rotation: Vector2f);
    fn get_rotation(&self) -> Vector2f;
    fn set_trans(&mut self, trans: Matrix4f);
    fn get_trans(&self) -> Matrix4f;
    fn set_offset(&mut self, offset: Matrix4f);
    fn get_offset(&self) -> Matrix4f;

}

pub trait SizedNode2D : Node2D {

    fn set_size(&mut self, size: Vector2f);
    fn get_size(&self) -> Vector2f;
    fn set_rect(&mut self, rect: Rect2f);
    fn get_rect(&self) -> Rect2f;

}

pub trait SizedNodeImplementor2D {

    fn get_fixed_size(&self) -> Vector2f {
        return Vector2f::new(1.0, 1.0);
    }

}

impl <T: NodeImplementor2D + SizedNodeImplementor2D> SizedNode2D for T where T: NodeImplementor2D + SizedNodeImplementor2D {

    fn set_size(&mut self, size: Vector2f) {
        let fs: Vector2f = self.get_fixed_size();
        if fs.x == 0.0 || fs.y == 0.0 {
            panic!("Cannot set the scaled size of an object with a fixed size of 0.")
        } else {
            self.set_scale(Vector2f { x: size.x / fs.x, y: size.y / fs.y });
        }
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }

    fn get_size(&self) -> Vector2f {
        return Vector2f { x: self.get_fixed_size().x * self.get_scale().x, y: self.get_fixed_size().y * self.get_scale().y };
    }

    fn set_rect(&mut self, rect: Rect2f) {

        self.set_pos(Vector2f { x: rect.x, y: rect.y });
        self.set_size(Vector2f { x: rect.width, y: rect.height });
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);

    }

    fn get_rect(&self) -> Rect2f {
        return Rect2f { x: self.get_pos().x, y: self.get_pos().y, width: self.get_size().x, height: self.get_size().y };
    }

}

/// Provides to multiply
pub trait NodeImplementor2D {

    fn get_node_obj_mut(&mut self) -> &mut NodeObject2D;

    fn get_node_obj(&self) -> &NodeObject2D;

    fn offset_children(&mut self, offset: Matrix4f) {

    }

}

impl <T: NodeImplementor2D> Node2D for T where T: NodeImplementor2D {

    fn set_pos(&mut self, pos: Vector2f) {
        self.get_node_obj_mut().set_pos(pos);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_pos(&self) -> Vector2f {
        return self.get_node_obj().get_pos();
    }
    fn set_scale(&mut self, scale: Vector2f) {
        self.get_node_obj_mut().set_scale(scale);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_scale(&self) -> Vector2f {
        return self.get_node_obj().get_scale();
    }
    fn set_rotation(&mut self, rotation: Vector2f) {
        self.get_node_obj_mut().set_rotation(rotation);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_rotation(&self) -> Vector2f {
        return self.get_node_obj().get_rotation();
    }
    fn set_trans(&mut self, trans: Matrix4f) {
        self.get_node_obj_mut().set_trans(trans);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_trans(&self) -> Matrix4f {
        return self.get_node_obj().get_trans();
    }
    fn set_offset(&mut self, offset: Matrix4f) {
        self.get_node_obj_mut().set_offset(offset);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_offset(&self) -> Matrix4f {
        return self.get_node_obj().get_offset();
    }

}

pub trait Node3D {

    fn set_pos(&mut self, pos: Vector3f);
    fn get_pos(&self) -> Vector3f;
    fn set_scale(&mut self, scale: Vector3f);
    fn get_scale(&self) -> Vector3f;
    fn set_rotation(&mut self, rotation: Vector3f);
    fn get_rotation(&self) -> Vector3f;
    fn set_trans(&mut self, trans: Matrix4f);
    fn get_trans(&self) -> Matrix4f;
    fn set_offset(&mut self, offset: Matrix4f);
    fn get_offset(&self) -> Matrix4f;

}

pub trait NodeImplementor3D {

    fn get_node_obj(&self) -> &NodeObject3D;

    fn get_node_obj_mut(&mut self) -> &mut NodeObject3D;

    fn offset_children(&mut self, offset: Matrix4f) {

    }

}

impl <T: NodeImplementor3D> Node3D for T where T: NodeImplementor3D {

    fn set_pos(&mut self, pos: Vector3f) {
        self.get_node_obj_mut().set_pos(pos);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }

    fn get_pos(&self) -> Vector3f {
        return self.get_node_obj().get_pos();
    }

    fn set_scale(&mut self, scale: Vector3f) {
        self.get_node_obj_mut().set_scale(scale);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }

    fn get_scale(&self) -> Vector3f {
        return self.get_node_obj().get_scale();
    }

    fn set_rotation(&mut self, rotation: Vector3f) {
        self.get_node_obj_mut().set_rotation(rotation);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_rotation(&self) -> Vector3f {
        return self.get_node_obj().get_rotation();
    }
    fn set_trans(&mut self, trans: Matrix4f) {
        self.get_node_obj_mut().set_trans(trans);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_trans(&self) -> Matrix4f {
        return self.get_node_obj().get_trans();
    }
    fn set_offset(&mut self, offset: Matrix4f) {
        self.get_node_obj_mut().set_offset(offset);
        let offset: Matrix4f = self.get_trans();
        self.offset_children(offset);
    }
    fn get_offset(&self) -> Matrix4f {
        return self.get_node_obj().get_offset();
    }

}

pub struct NodeObject3D {

    pub trans: Matrix4f,
    pub rotation: Vector3f,
    pub offset: Matrix4f,
}

impl NodeObject3D {

    pub fn new() -> NodeObject3D {

        return NodeObject3D { trans: Matrix4f::identity(), rotation: Vector3f::new(0.0, 0.0, 0.0), offset: Matrix4f::identity() };

    }

}

impl NodeObject3D {

    pub fn set_pos(&mut self, pos: Vector3f) {
        self.trans.set_translation(pos);
    }

    pub fn get_pos(&self) -> Vector3f {
        return self.trans.get_translation();
    }

    pub fn set_scale(&mut self, scale: Vector3f) {
        self.trans.set_scale(scale);
    }

    pub fn get_scale(&self) -> Vector3f {
        return self.trans.get_scale();
    }

    pub fn set_rotation(&mut self, rotation: Vector3f) {
        self.rotation = rotation;
    }

    pub fn get_rotation(&self) -> Vector3f {
        return self.rotation;
    }

    pub fn set_trans(&mut self, trans: Matrix4f) {
        self.trans = trans;
    }

    pub fn get_trans(&self) -> Matrix4f {
        let rx = Matrix4f::from_angle_x(cgmath::Rad(self.rotation.x));
        let ry = Matrix4f::from_angle_y(cgmath::Rad(self.rotation.y));
        let rz = Matrix4f::from_angle_z(cgmath::Rad(self.rotation.z));

        let rotation: Matrix4f = ry * rx * rz;
        return self.offset * self.trans * rotation;
    }

    pub fn set_offset(&mut self, offset: Matrix4f) {
        self.offset = Matrix4f::from_translation(offset.get_translation());
    }

    pub fn get_offset(&self) -> Matrix4f {
        return self.offset;
    }
}

/// The render node structure is a container structure which holds a render component as a node.
/// This allows it to be manipulated as if it is a node, and can be used within a scene.
pub struct ContainerNode3D<T> {

    pub node: NodeObject3D,
    pub component: T,

}

impl<T> ContainerNode3D<T> {

    /// Creates a new 3D render object container with the specified component.
    /// The node is set to 0 by default for all position and rotation values, and it is scaled to 1.
    pub fn new(component: T) -> Self {
        return Self { node: NodeObject3D::new(), component };
    }

}

impl<T> NodeImplementor3D for ContainerNode3D<T> {

    fn get_node_obj(&self) -> &NodeObject3D {
        return &self.node;
    }

    fn get_node_obj_mut(&mut self) -> &mut NodeObject3D {
        return &mut self.node;
    }

}

impl<T> Deref for ContainerNode3D<T> {

    type Target = T;

    fn deref(&self) -> &T {
        return &self.component;
    }

}

impl<T> DerefMut for ContainerNode3D<T> {

    fn deref_mut(&mut self) -> &mut T {
        return &mut self.component;
    }

}