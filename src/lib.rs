#![feature(copy_within)]
#![feature(duration_float)]

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
pub extern crate cgmath;
pub extern crate specs;
pub extern crate specs_hierarchy;
pub extern crate winit;

pub mod types;
pub use types::*;

#[macro_use]
pub mod core;
pub mod render;
pub mod command;
pub mod pipeline;
pub mod window;
pub mod buffer;
pub mod texture;
pub mod input;

pub mod node;
pub mod physics;

pub mod scene;

pub mod spatial;

pub mod script;

pub mod app;

pub use crate::core::LogExpect;