//#[cfg(windows)]
//pub extern crate gfx_backend_dx12 as backend;
//#[cfg(target_os = "macos")]
//extern crate gfx_backend_metal as backend;
//#[cfg(all(unix, not(target_os = "macos")))]
extern crate gfx_backend_vulkan as backend;

pub use backend::Backend;
pub extern crate gfx_hal as gfx;
pub extern crate assimp_sys as ai;
pub extern crate libc;
extern crate cgmath;
pub extern crate specs;

#[macro_use]
pub mod core;
pub mod render;
pub mod window;
pub mod buffer;
pub mod command;
pub mod pipeline;
pub mod node;
pub mod physics;

pub mod texture;

pub mod component;
pub mod spatial;

pub mod scene;

pub mod input;

pub mod types;

pub use types::*;

pub use core::LogExpect;