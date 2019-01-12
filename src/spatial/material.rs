use crate::*;

pub struct MaterialComponent {
    materials: MaterialsList,
    pub buffer: buffer::SharedBuffer,
    should_update_buffer: bool,
}

impl MaterialComponent {
    pub fn new(materials: MaterialsList, device: &mut core::Device) -> Self {
        let buffer: buffer::Buffer = buffer::Buffer::alloc_uniform(&[materials], device);
        return Self { materials, buffer: buffer::SharedBuffer::new(buffer), should_update_buffer: false };
    }
    pub fn add_material(&mut self, material: Material, device: &mut core::Device) {
        self.materials.add_material(material);
        self.should_update_buffer = true;
    }
    pub fn materials(&self) -> &MaterialsList {
        return &self.materials;
    }
    pub fn materials_mut(&mut self) -> &mut MaterialsList {
        self.should_update_buffer = true;
        return &mut self.materials;
    }
}

impl specs::Component for MaterialComponent {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Material {

    pub ambient: Al16<OpaqueColor>,
    pub diffuse: Al16<OpaqueColor>,
    pub specular: Al16<OpaqueColor>,
    pub roughness: f32,

}

impl Material {

    pub fn new(ambient: OpaqueColor, diffuse: OpaqueColor, specular: OpaqueColor, roughness: f32) -> Self {
        return Self {
            ambient: Al16::new(ambient),
            diffuse: Al16::new(diffuse),
            specular: Al16::new(specular),
            roughness
        };
    }



    pub fn with_color(color: OpaqueColor, roughness: f32) -> Self {
        return Self::new(color, color, color, roughness);
    }

}
impl Default for Material {
    fn default() -> Self {
        return Self::with_color(OpaqueColor::white(), 0.8);
    }
}
const MAX_MATERIALS: usize = 20;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MaterialsList {

    pub count: Al16<i32>,
    pub materials: [Material; MAX_MATERIALS],

}

impl MaterialsList {

    pub fn new() -> Self {
        return Self { count: Al16::new(0), materials: [Material::default(); MAX_MATERIALS] };
    }

    pub fn add_material(&mut self, data: Material) {
        if (*self.count as usize) < MAX_MATERIALS {
            self.materials[*self.count as usize] = data;
        }
        *self.count += 1;
    }

    pub fn remove_material(&mut self, index: usize) -> Material {

        assert!(index >= *self.count as usize, "Index out of range for remove operation.");

        let data: Material = self.materials[index];
        if index < *self.count as usize - 1 {
            self.materials.copy_within(index + 1..*self.count as usize - 1, index);
        }

        self.materials[*self.count as usize] = Material::default();
        *self.count -= 1;

        return data;
    }

}