use crate::*;

pub trait Light {

    fn get_ambient(&self) -> spatial::AmbientData {
        return spatial::AmbientData::new(0.0, Color::black());
    }

    fn get_diffuse(&self) -> spatial::DiffuseData {
        return spatial::DiffuseData::new(0.0, Color::black(), Vector3f::zero());
    }

    fn get_data(&self) -> spatial::LightData {
        return spatial::LightData { ambient: self.get_ambient(), diffuse: self.get_diffuse() };
    }

}

/// Represents a directional light which emits from a direction rather than a point.
/// A directional light can also emit ambient light using the 'ambiance' field.
/// This kind if light is less computationally intensive than point lights.
#[derive(Copy, Clone)]
pub struct Directional {

    pub intensity: f32,
    pub ambiance: f32,
    pub color: Color,
    pub direction: Vector3f,

}

impl Directional {

    pub fn new(intensity: f32, ambiance: f32, color: Color, direction: Vector3f) -> Directional {
        return Directional { intensity, ambiance, color, direction };
    }

    /// Creates a directional light with intensity and color approximately that of the sun and with the direction specified.
    pub fn create_sun(direction: Vector3f) -> Directional {
        return Directional { intensity: 0.8, ambiance: 0.4, color: Color::new(1.0, 0.88, 0.48, 1.0), direction };
    }

}

impl Light for Directional {

    fn get_ambient(&self) -> spatial::AmbientData {
        return spatial::AmbientData::new(self.ambiance, self.color);
    }

    fn get_diffuse(&self) -> spatial::DiffuseData {
        return spatial::DiffuseData::new(self.intensity, self.color, self.direction);
    }

}