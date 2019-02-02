use super::*;

extern crate colored;

use std::sync::Arc;
use std::io::Read;
use std::rc::Rc;

use gfx::PhysicalDevice;
use gfx::Surface;
use gfx::Device as GfxDevice;
use gfx::Instance as GfxInstance;
use gfx::Swapchain;
use gfx::DescriptorPool;

use self::colored::{Color as PrintColor, Colorize as PrintColorize, ColoredString as PrintColoredString};

/// The instance object for the engine which all other devices are created from.
/// This structure encapsulates the backend instance object (e.g. vulkan instance).
/// It is therefore only needed for device creation.
pub struct Instance  {
    pub gfx_inst: Arc<backend::Instance>,
}

impl Instance {

    /// Creates a new instance object with the specified application name.
    /// The name of the application is backend specific and may map to different things.
    pub fn create(application_name: & str) -> Self {
        let gfx_inst = Arc::new(backend::Instance::create(application_name, 1));
        return Self { gfx_inst };
    }

}

/// The device structure which contains data about a device.
/// This contains surface data, physical and logical device data as well as graphics queues.
/// This structure must be passed to most graphics objects during initialisation.
pub struct Device {

    pub color_format: gfx::format::Format,
    pub adapter: gfx::Adapter<Backend>,
    pub gpu: Arc<<Backend as gfx::Backend>::Device>,
    pub queue_group: gfx::QueueGroup<Backend, gfx::Graphics>,

    pub capabilites: gfx::SurfaceCapabilities,

}

impl Device {

    /// Creates a new device instance using the specified instance and window.
    /// This device can be used to create practically every graphics object.
    pub fn create(instance: &core::Instance, window_surface: &window::WindowSurface) -> Device {
        // Just select the first device
        // TODO: Amend code to check if device is suitable.
        let mut adapter: gfx::Adapter<Backend> = instance.gfx_inst.enumerate_adapters().remove(0);

        let (gpu, queue_group) =
            adapter.open_with::<_, gfx::Graphics>(1, |family| window_surface.surface.supports_queue_family(family))
                .expect("Fatal Error: Failed to find valid device.");

        // We want to get the capabilities (`caps`) of the surface, which tells us what
        // parameters we can use for our swapchain later. We also get a list of supported
        // image formats for our surface.
        let (caps, formats, _, _) = window_surface.surface.compatibility(&adapter.physical_device);

        let color_format = {
            // We must pick a color format from the list of supported formats. If there
            // is no list, we default to Rgba8Srgb.
            match formats {
                Some(choices) => choices
                    .into_iter()
                    .find(|format| format.base_format().1 == gfx::format::ChannelType::Srgb)
                    .unwrap(),
                None => gfx::format::Format::Rgba8Srgb,
            }
        };

        return Device { color_format, adapter: adapter, gpu: Arc::new(gpu), queue_group, capabilites: caps };

    }

    pub fn load_shader(&self, path: &str) -> Result<<Backend as gfx::Backend>::ShaderModule, &'static str> {

        if let Ok(mut f) = std::fs::File::open(path) {
            let mut contents = String::new();
            if let Ok(_) = f.read_to_string(&mut contents) {
                return Ok(unsafe { self.gpu.create_shader_module(&contents.into_bytes()).unwrap() });
            } else {
                return Err("Failed to read shader file.");
            }
        } else {
            return Err("Failed to open shader file. Does it exist?");
        }

    }

    pub fn load_shader_raw(&self, bytes: &[u8]) -> Result<<Backend as gfx::Backend>::ShaderModule, &'static str> {
        if let Ok(module) = unsafe { self.gpu.create_shader_module(bytes) } {
            return Ok(module);
        } else {
            return Err("Failed to create shader module.");
        }
    }

    pub fn upload_type_for(&self, unbound_buffer: &<Backend as gfx::Backend>::Buffer, properties: gfx::memory::Properties) -> (gfx::MemoryTypeId, gfx::memory::Requirements) {
        let memory_types = self.adapter.physical_device.memory_properties().memory_types;

        let req = unsafe { self.gpu.get_buffer_requirements(unbound_buffer) };

        let upload_type = memory_types
            .iter()
            .enumerate()
            .find(|(id, ty)| {
                let type_supported = req.type_mask & (1_u64 << id) != 0;
                type_supported && ty.properties.contains(properties)
            }).map(|(id, _ty)| gfx::adapter::MemoryTypeId(id))
            .expect("Could not find appropriate vertex buffer memory type.");
        return (upload_type, req);
    }

    pub fn create_token(&self) -> DeviceToken {
        return DeviceToken::create(self);
    }

}

/// A structure which contains device data which can be used to destroy objects.
/// This token can be used to destroy objects.
pub struct DeviceToken {

    pub gpu: Arc<<Backend as gfx::Backend>::Device>,

}

impl DeviceToken {

    /// Creates a device token from a device.
    pub fn create(device: &Device) -> Self {
        return Self { gpu: device.gpu.clone() };
    }

}

pub static mut PRINT_LOG: bool = true;
pub static mut PRINT_DEBUG: bool = true;
pub static mut PRINT_ERR: bool = true;
pub static mut PRINT_PANIC: bool = true;

pub static mut DEBUG_VERBOSITY: u32 = 0;

#[macro_export]
macro_rules! log {
    (msg, $($arg:tt)*) => ({
        $crate::core::log(format!($($arg)*));
    });
    (debug, $verb:expr, $($arg:tt)*) => ({
        $crate::core::log_debug(format!($($arg)*), $verb);
    });
    (temp, $($arg:tt)*) => ({
        $crate::core::log_temp(format!($($arg)*));
    });
    (err, $($arg:tt)*) => ({
        $crate::core::log_err(format!($($arg)*));
    });
    (panic, $($arg:tt)*) => ({
        $crate::core::log_panic(format!($($arg)*));
    })
}
pub fn log(data: String) {
    if unsafe { PRINT_LOG } {
        println!("{} {}", "MESSAGE LOG:".bold().blue(), data.blue());
    }

}
pub fn log_debug(data: String, verbosity: u32) {
    if unsafe { PRINT_DEBUG } && unsafe { verbosity <= DEBUG_VERBOSITY } {
        println!("{} {}", "DEBUG LOG:".bold().green(), data.green());
    }
}
pub fn log_temp(data: String) {
    if unsafe { PRINT_DEBUG } {
        println!("{} {}", "TEMPORARY LOG:".bold().yellow(), data.yellow());
    }
}
pub fn log_err(data: String) {
    if unsafe { PRINT_ERR } {
        println!("{} {}", "ERROR LOG:".bold().red(), data.red());
    }
}
pub fn log_panic(data: String) {
    if unsafe { PRINT_PANIC } {
        println!("{} {}", "PANIC LOG:".bold().bright_red(), data.bright_red());
    }
    panic!("Engine panic at 'log(panic)': {}", data);
}

pub trait LogExpect {
    type Out;
    fn log_expect(self, err: &str) -> Self::Out;
}

impl<T, E> LogExpect for Result<T, E> {
    type Out = T;
    fn log_expect(self, err: &str) -> T {
        if let Ok(t) = self {
            return t;
        } else {
            log_panic(err.to_string());
        }
        panic!("");
    }
}

impl<T> LogExpect for Option<T> {
    type Out = T;
    fn log_expect(self, err: &str) -> T {
        if let Some(t) = self {
            return t;
        } else {
            log_panic(err.to_string());
        }
        panic!("");
    }
}
