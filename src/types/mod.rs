use std;

pub use cgmath::*;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use std::ptr;

// Using 64 bit types for standard vectors.
// This can be changed to use 32 bit types if memory is limited.
pub type Vector2f = Vector2<f32>;
pub type Vector2i = Vector2<i32>;
pub type Vector2u = Vector2<u32>;

pub type Vector3f = Vector3<f32>;
pub type Vector3i = Vector3<i32>;
pub type Vector3u = Vector3<u32>;

pub type Vector4f = Vector4<f32>;
pub type Vector4i = Vector4<i32>;
pub type Vector4u = Vector4<u32>;

pub type Matrix4f = Matrix4<f32>;
pub type Matrix4i = Matrix4<i32>;
pub type Matrix4u = Matrix4<u32>;


pub trait ToVec3f {

    fn to_vec3(&self) -> Vector3f;

}

pub trait ToVec2f {

    fn to_vec2(&self) -> Vector2f;

}

impl ToVec3f for Vector2f {
    fn to_vec3(&self) -> Vector3f {
        return Vector3f { x: self.x, y: self.y, z: 0.0 };
    }
}

impl ToVec2f for Vector3f {
    fn to_vec2(&self) -> Vector2f {
        return Vector2f { x: self.x, y: self.y };
    }
}

#[derive(Copy, Clone)]
pub struct Color {

    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32

}

impl Color {

    pub fn new(r: f32, b: f32, g: f32, a: f32) -> Color {
        return Color { r, g, b, a }
    }
    pub fn black() -> Color {
        return Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
    pub fn white() -> Color {
        return Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
    pub fn red() -> Color {
        return Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    }
    pub fn green() -> Color {
        return Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    }
    pub fn blue() -> Color {
        return Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
    }
    pub fn yellow() -> Color {
        return Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 }
    }
    pub fn zero() -> Color {
        return Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    pub fn invert(&self) -> Color {
        return Color { r: 1.0 - self.r, g: 1.0 - self.g, b: 1.0 - self.b, a: 1.0 };
    }
    pub fn to_raw_color(&self) -> [f32; 4] {
        return [self.r, self.g, self.b, self.a];
    }

}

#[derive(Copy, Clone)]
pub struct Rect<T: std::marker::Copy + std::ops::Add<Output = T> + std::cmp::PartialOrd> {

    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,

}

impl<T: std::marker::Copy + std::ops::Add<Output = T> + std::cmp::PartialOrd> Rect<T> {

    pub fn new(x: T, y: T, width: T, height: T) -> Rect<T> {

        return Rect { x, y, width, height  };

    }

    pub fn intersects(&self, rect: Rect<T>) -> bool {
        if rect.x + rect.width > self.x && rect.x < self.x + self.width {
            if rect.y + rect.height > self.y && rect.y < self.y + self.height {

                return true;
            }
        }
        return false;
    }

    pub fn get_pos(&self) -> Vector2<T> {
        return Vector2::new(self.x, self.y);
    }

    pub fn get_size(&self) -> Vector2<T> {
        return Vector2::new(self.width, self.height);
    }

}

pub type Rect2f = Rect<f32>;
pub type Rect2u = Rect<u32>;

pub trait Translatable {

    fn set_translation(&mut self, translation: Vector3f);

    fn get_translation(&self) -> Vector3f;

    fn get_negative_translation(&self) -> Matrix4f;

    fn set_scale(&mut self, scale: Vector3f);

    fn get_scale(&self) -> Vector3f;
}

pub trait Mat4fData {

    fn get_data(&self) -> [[f32; 4]; 4];

}

impl Mat4fData for Matrix4f {

    fn get_data(&self) -> [[f32; 4]; 4] {
        return [
            [self.x.x, self.x.y, self.x.z, self.x.w],
            [self.y.x, self.y.y, self.y.z, self.y.w],
            [self.z.x, self.z.y, self.z.z, self.z.w],
            [self.w.x, self.w.y, self.w.z, self.w.w],
        ];
    }

}

impl Translatable for Matrix4f {

    fn set_translation(&mut self, translation: Vector3f) {

        self.w.x = translation.x;
        self.w.y = translation.y;
        self.w.z = translation.z;

    }

    fn get_translation(&self) -> Vector3f {
        return Vector3f { x: self.w.x, y: self.w.y, z: self.w.z };
    }

    fn get_negative_translation(&self) -> Matrix4f {

        let mut mat: Matrix4f = Matrix4f::identity();
        let translate: Vector3f = self.get_translation();

        mat.w.x = -translate.x;
        mat.w.y = -translate.y;
        mat.w.z = -translate.z;

        return mat;

    }

    fn set_scale(&mut self, scale: Vector3f) {

        self.x.x = scale.x;
        self.y.y = scale.y;
        self.z.z = scale.z;

    }

    fn get_scale(&self) -> Vector3f {
        return Vector3f { x: self.x.x, y: self.y.y, z: self.z.z };
    }

}

/**
This enum can hold any different type of heap memory (ie. Box, Rc, Arc, RefCell etc.)
*/
pub enum Heap<T> {
    Box(std::boxed::Box<T>),
    Rc(std::rc::Rc<T>),
    Arc(std::sync::Arc<T>),
}

impl<T> std::convert::AsRef<T> for Heap<T> {
    fn as_ref(&self) -> &T {
        match self {
            Heap::Box(v) => v.as_ref(),
            Heap::Rc(v) => v.as_ref(),
            Heap::Arc(v) => v.as_ref(),
        }
    }
}

impl<T> std::ops::Deref for Heap<T> {
    type Target = T;
    fn deref(&self) -> &T {
        return self.as_ref();
    }
}

/// This type allows for different allocation depending on requirements.
/// For example, in some cases, data might need to be shared to increase efficiency.
/// In this case, a Heap(Rc()) can be used.
/// In other cases it may be more optimal to store by value rather than by reference, so a Val() can be used.
pub enum Resource<T> {
    Heap(Heap<T>),
    Val(T),
}

impl<T> std::convert::AsRef<T> for Resource<T> {
    fn as_ref(&self) -> &T {
        match self {
            Resource::Heap(h) => return h.as_ref(),
            Resource::Val(v) => return &v,
        }
    }
}

impl<T> std::ops::Deref for Resource<T> {
    type Target = T;
    fn deref(&self) -> &T {
        return self.as_ref();
    }
}

pub trait Identified {

    fn get_id(&self) -> u32;

}

/// This structure represents a handle to an imperium_core object.
/// This can be any object, and should be used as follows:
/// The index should be the last known index of the array in which the object is contained.
/// The id should match the id stored in the component that this handle references.
/// When the lookup takes place, the component at the index held in the 'index' field is checked.
/// If the id of the component matches the id of the handle, that component is returned.
/// If not, the surrounding elements are checked and if the id is found, the component is returned and the index is updated.
#[derive(Clone)]
pub struct Handle {

    /// The index pointer points to the index of this handle object.
    index_ptr: Rc<usize>,

}

impl Handle {

    pub fn create(index: usize) -> Self {
        return Self { index_ptr: Rc::new(index) };
    }

    pub fn new(index_ptr: Rc<usize>) -> Self {
        return Self { index_ptr };
    }

    pub fn get_index(&self) -> usize {
        return *self.index_ptr;
    }

    pub fn is_valid(&self) -> bool {
        if self.get_index() == std::usize::MAX {
            return false;
        }
        return true;
    }

    /// Sets the index of the handle.
    /// This is unsafe because we convert the immutable reference from the Rc object to a mutable pointer then write to it.
    /// However, if the handle is used as it should be, this should not cause problems due to the handle only being accessed from the object that changes it.
    unsafe fn set_index(&self, index: usize) {
        let iptr: *mut usize = self.index_ptr.as_ref() as *const usize as *mut usize;
        *iptr = index;
    }

    unsafe fn invalidate(&self) {
        let iptr: *mut usize = self.index_ptr.as_ref() as *const usize as *mut usize;
        *iptr = std::usize::MAX;
    }

}



impl std::cmp::PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        if self.index_ptr == other.index_ptr {
            return true;
        }
        return false;
    }
}

/// This is the container struct for handled object data.
/// Data of a specific type can be added and handles will be created.
/// That data can then be accessed using the handle.
pub struct HandledData<T> {

    pub data: Vec<HandledObject<T>>,

}

impl<T> HandledData<T> {

    pub fn new() -> Self {
        return Self { data: Vec::new() };
    }

    pub fn with_capacity(capacity: usize) -> Self {
        return Self { data: Vec::with_capacity(capacity) };
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        let mut data: Vec<HandledObject<T>> = Vec::with_capacity(vec.len());
        let mut i = 0;
        for e in vec {
            data.push(HandledObject::new(e, i));
            i += 1;
        }
        return Self { data };
    }

    pub fn len(&self) -> usize {
        return self.data.len();
    }

    /// Adds an object with the specified value to the dynamic array.
    /// A handle will be returned which can be used to access this object.
    pub fn push(&mut self, value: T) -> Handle {
        let index: usize = self.data.len();
        let handled_object: HandledObject<T> = HandledObject::new(value, index);
        let handle: Handle = handled_object.handle.clone();
        self.data.push(handled_object);
        return handle;
    }

    /// Removes a the object with the specified handle.
    /// This function does contain internal calls to unsafe code, but if used properly, everything should function safely.
    pub fn remove(&mut self, handle: Handle) -> T {
        let index: usize = handle.get_index();
        let result = self.data.remove(index);
        unsafe { result.handle.invalidate() };
        if index < self.data.len() {
            for i in index..self.data.len() {
                unsafe { self.data[i].handle.set_index(i) };
            }
        }
        return result.value;
    }

    pub fn get_handled_access<'a>(&'a mut self, handle: Handle) -> Option<(&'a mut T, HandledDataAccess<'a, T>)> {
        let vec_unsafe: &'a mut Vec<HandledObject<T>> = unsafe { mem::transmute(&mut self.data) };
        if let Some(data) = self.get_mut(handle.clone()) {
            let access: HandledDataAccess<'a, T> = HandledDataAccess::new(vec_unsafe, handle.get_index());
            return Some((data , access));
        }
        return None;
    }

    pub fn get_handles(&self) -> Vec<Handle> {
        let mut result: Vec<Handle> = Vec::with_capacity(self.data.len());
        let mut i = 0;
        for item in self.data.iter() {
            result.push(Handle::create(i));
            i += 1;
        }
        return result;
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        let index = handle.get_index();
        if let Some(data) = self.data.get(index) as Option<&HandledObject<T>> {
            return Some(&data.value);
        }
        return None;
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let index = handle.get_index();
        if let Some(data) = self.data.get_mut(index) as Option<&mut HandledObject<T>> {
            return Some(&mut data.value);
        }
        return None;
    }

    pub fn iter(&self) -> std::slice::Iter<HandledObject<T>> {
        return self.data.iter();
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<HandledObject<T>> {
        return self.data.iter_mut();
    }

}

pub struct HandledDataAccess<'a, T> {

    data: &'a mut Vec<HandledObject<T>>,
    excl: usize,
    iter: usize,

}

impl<'a, T> HandledDataAccess<'a, T> {

    pub fn new(data: &'a mut Vec<HandledObject<T>>, excl: usize) -> Self {
        return Self { data, excl, iter: 0 };
    }

}

/// This is a container struct which holds any object with a specific identifier attached.
pub struct HandledObject<T> {

    pub value: T,
    pub handle: Handle,

}

impl<T> HandledObject<T> {

    pub fn new(value: T, index: usize) -> Self {
        return Self { value, handle: Handle::create(index) };
    }

}

impl<T> std::ops::Deref for HandledObject<T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        return &self.value;
    }

}

impl<T> std::ops::DerefMut for HandledObject<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.value;
    }

}

/// This struct provides an extremely unsafe way to access a pointer.
/// It is very primitive in that it has to be manually invalidated if the object it references is destroyed.
pub struct UnsafeAccess<T> {

    valid: bool,
    raw: *mut T,

}

impl<T> UnsafeAccess<T> {

    /// Creates a new unsafe access to a value.
    /// This is completely unsafe. If the
    pub fn new(raw: *mut T) -> Self {
        return Self { valid: true, raw };
    }

    pub const fn invalid() -> Self {
        return Self { valid: false, raw: ptr::null_mut() };
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub unsafe fn get(&self) -> Option<&T> {
        if self.valid {
            return Some(& *self.raw);
        } else {
            return None;
        }
    }

    pub unsafe fn get_mut(&mut self) -> Option<&mut T> {
        if self.valid {
            return Some(&mut *self.raw);
        } else {
            return None;
        }
    }

}

