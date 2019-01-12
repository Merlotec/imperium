use crate::*;

use std::ops::Deref;
use std::ops::DerefMut;

pub const UNBOUND_FIXED_SIZE: Vector2f = Vector2f { x: 10.0, y: 10.0 };

pub trait Node {

    fn set_trans(&mut self, trans: Matrix4f);
    fn get_trans(&self) -> Matrix4f;
    fn set_offset(&mut self, offset: Matrix4f);
    fn get_offset(&self) -> Matrix4f;

}


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

impl Node for NodeObject2D {

    fn set_trans(&mut self, trans: Matrix4f) {
        self.trans = trans;
    }
    fn get_trans(&self) -> Matrix4f {
        let rx = Matrix4f::from_angle_x(cgmath::Rad(self.rotation.x));
        let ry = Matrix4f::from_angle_z(cgmath::Rad(self.rotation.y));

        let rotation: Matrix4f = rx * ry;
        return self.offset * self.trans * rotation;
    }
    fn set_offset(&mut self, offset: Matrix4f) {
        // The inherited offset can only be a translation otherwise there will be issues with size and scaling.
        self.offset = Matrix4f::from_translation(offset.get_translation());
    }
    fn get_offset(&self) -> Matrix4f {
        return self.offset;
    }

}

impl Node2D for NodeObject2D {

    fn set_pos(&mut self, pos: Vector2f) {
        self.trans.set_translation(pos.to_vec3());
    }
    fn get_pos(&self) -> Vector2f {
        return self.trans.get_translation().to_vec2();
    }
    fn set_scale(&mut self, scale: Vector2f) {
        self.trans.set_scale(scale.to_vec3());
    }
    fn get_scale(&self) -> Vector2f {
        return self.trans.get_scale().to_vec2();
    }
    fn set_rotation(&mut self, rotation: Vector2f) {
        self.rotation = rotation;
    }
    fn get_rotation(&self) -> Vector2f {
        return self.rotation;
    }

}

pub trait Node2D : Node {

    fn set_pos(&mut self, pos: Vector2f);
    fn get_pos(&self) -> Vector2f;
    fn set_scale(&mut self, scale: Vector2f);
    fn get_scale(&self) -> Vector2f;
    fn set_rotation(&mut self, rotation: Vector2f);
    fn get_rotation(&self) -> Vector2f;

}

pub trait SizedNode2D : Node2D {

    fn set_size(&mut self, size: Vector2f);
    fn get_size(&self) -> Vector2f;
    fn set_rect(&mut self, rect: Rect2f);
    fn get_rect(&self) -> Rect2f;

}

impl specs::Component for NodeObject2D {
    type Storage = specs::DenseVecStorage<Self>;
}

pub trait Node3D : Node {

    fn set_pos(&mut self, pos: Vector3f);
    fn get_pos(&self) -> Vector3f;
    fn set_scale(&mut self, scale: Vector3f);
    fn get_scale(&self) -> Vector3f;
    fn set_rotation(&mut self, rotation: Vector3f);
    fn get_rotation(&self) -> Vector3f;

}

#[derive(Copy, Clone)]
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

impl Node for NodeObject3D {

    fn set_trans(&mut self, trans: Matrix4f) {
        self.trans = trans;
    }

    fn get_trans(&self) -> Matrix4f {
        let rx = Matrix4f::from_angle_x(cgmath::Rad(self.rotation.x));
        let ry = Matrix4f::from_angle_y(cgmath::Rad(self.rotation.y));
        let rz = Matrix4f::from_angle_z(cgmath::Rad(self.rotation.z));

        let rotation: Matrix4f = ry * rx * rz;
        return self.offset * self.trans * rotation;
    }

    fn set_offset(&mut self, offset: Matrix4f) {
        self.offset = Matrix4f::from_translation(offset.get_translation());
    }

    fn get_offset(&self) -> Matrix4f {
        return self.offset;
    }

}

impl Node3D for NodeObject3D {

    fn set_pos(&mut self, pos: Vector3f) {
        self.trans.set_translation(pos);
    }

    fn get_pos(&self) -> Vector3f {
        return self.trans.get_translation();
    }

    fn set_scale(&mut self, scale: Vector3f) {
        self.trans.set_scale(scale);
    }

    fn get_scale(&self) -> Vector3f {
        return self.trans.get_scale();
    }

    fn set_rotation(&mut self, rotation: Vector3f) {
        self.rotation = rotation;
    }

    fn get_rotation(&self) -> Vector3f {
        return self.rotation;
    }

}

impl specs::Component for NodeObject3D {
    type Storage = specs::DenseVecStorage<Self>;
}