use crate::*;

use std::ops::Deref;
use std::ops::DerefMut;

pub trait Light {

    fn get_data(&self, pos: Vector3f) -> LightData;

}

pub struct LightComponent {
    pub light: Box<Light + Send + Sync>,
}

impl LightComponent {
    pub fn new<L: 'static + Light + Send + Sync>(light: L) -> Self {
        return Self { light: Box::new(light) };
    }
}

impl specs::Component for LightComponent {
    type Storage=specs::VecStorage<Self>;
}

/// Represents a directional light which emits from a direction rather than a point.
/// A directional light can also emit ambient light using the 'ambiance' field.
/// This kind if light is less computationally intensive than point lights.
#[derive(Copy, Clone)]
pub struct PointLight {

    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub color: OpaqueColor,

}

impl PointLight {

    pub fn new(ambient: f32, diffuse: f32, specular: f32, color: OpaqueColor) -> Self {
        return Self { ambient, diffuse, specular, color };
    }

    /// Creates a directional light with intensity and color approximately that of the sun and with the direction specified.
    pub fn create_sun() -> Self {
        return Self::new(0.5, 0.8, 0.5, OpaqueColor::new(1.0, 0.88, 0.48));
    }

}

impl Light for PointLight {

    fn get_data(&self, pos: Vector3f) -> LightData {
        return LightData::new(pos, self.color, self.ambient, self.diffuse, self.specular);
    }

}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LightData {
    pub pos: Al16<Vector3f>,
    pub color: OpaqueColor,
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
}

impl LightData {
    pub fn new(pos: Vector3f, color: OpaqueColor, ambient: f32, diffuse: f32, specular: f32) -> Self {
        return Self { pos: Al16::new(pos), color, ambient, diffuse, specular };
    }
}

impl Default for LightData {
    fn default() -> Self {
        return Self::new(Vector3f::zero(), OpaqueColor::black(), 0.0, 0.0, 0.0);
    }
}

const MAX_LIGHTS: usize = 20;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LightsList {

    pub count: Al16<i32>,
    pub lights: [LightData; MAX_LIGHTS],

}

impl LightsList {

    pub fn new() -> Self {
        return LightsList { count: Al16::new(0), lights: [LightData::default(); MAX_LIGHTS] };
    }

    pub fn add_light(&mut self, data: LightData) {
        if (*self.count as usize) < MAX_LIGHTS {
            self.lights[*self.count as usize] = data;
        }
        *self.count += 1;
    }

    pub fn remove_light(&mut self, index: usize) -> LightData {

        assert!(index >= *self.count as usize, "Index out of range for remove operation.");

        let data: LightData = self.lights[index];
        if index < *self.count as usize - 1 {
            self.lights.copy_within(index + 1..*self.count as usize - 1, index);
        }

        self.lights[*self.count as usize] = LightData::default();
        *self.count -= 1;

        return data;
    }

}

/// A struct which contains light data and controls the GPU buffer.
pub struct LightsController {
    pub lights: LightsList,
    pub buffer: buffer::SharedBuffer,
}

impl LightsController {

    pub fn new(device: &mut core::Device) -> Self {
        let lights: LightsList = LightsList::new();
        return Self { lights, buffer: buffer::SharedBuffer::new(buffer::Buffer::alloc_uniform(&[lights], device)) };
    }

    pub fn clone_buffer(&self) -> buffer::SharedBuffer {
        return self.buffer.clone();
    }

    pub fn add_light(&mut self, data: LightData) {
        self.lights.add_light(data);
    }

    pub fn remove_light(&mut self, index: usize) -> LightData {
        return self.lights.remove_light(index);
    }

    pub fn update_buffer(&self, device: &mut core::Device) {
        self.buffer.fill_buffer(&[self.lights], device);
    }

}