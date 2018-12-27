pub use super::*;

pub struct Emission {

    pub strength: f32,
    pub color: Color,

}

impl Emission {

    pub fn new(strength: f32, color: Color) -> Emission {
        return Emission { strength, color };
    }

    pub fn none() -> Emission {
        return Emission { strength: 0.0, color: Color::black() };
    }

}