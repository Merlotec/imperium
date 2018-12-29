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

/// This trait defines the core Imperium drawing abstraction.
/// The Imperium engine can be used by passing a single 'Component' implementing object to the engine object.
/// In most cases, the rendering method used (e.g. Scene2D, Scene3D) will implement this for the programmer.
/// These abstractions use there own methods of rendering their individual components.
/// This allows for modularity as every rendering method can pass its own arguments to it's components and are not restricted to what is defined in this trait.
pub trait Component {

    /// All logical behaviour (not direct rendering) should be handled in this function.
    /// The command pool object is available in this function in order to move buffers, upload images etc.
    /// This cannot be done in the render function because the command pool is borrowed by the encoder.
    /// The Graphics object contains the Renderer object and the CommandDispatch object.
    /// Both can be used to update buffers and submit GPU commands.
    /// Theoretically, this function could be used to draw but IT SHOULD NOT BE DONE UNLESS YOU REALLY KNOW WHAT YOU ARE DOING!
    /// This is because all rendering should be done using the same encoder submission.
    fn update(&mut self, renderer: &mut render::Renderer, window: &mut window::Window, delta: f32);

    /// This function is called frames where events should be handled.
    /// It is the responsibility of the implementation to defer calls to any children.
    fn handle_events(&mut self, events: &Vec<window::Event>, renderer: &mut render::Renderer, window: &mut window::Window, delta: f32) {}

    /// The draw functions should be added to the encoder in order to draw to the screen.
    /// Logic related to the component SHOULD NOT be updated in this function. For logic, use the 'update' function.
    /// This function does not have access to the command pool object so creating and moving buffers cannot be done.
    fn render(&mut self, graphics: &mut render::Graphics, encoder: &mut command::Encoder);

}

/// The instance object for the engine which all other devices are created from.
/// This structure encapsulates the backend instance object (e.g. vulkan instance).
/// It is therefore only needed for device creation.
pub struct Instance  {
    pub gfx_inst: Arc<backend::Instance>,
}

impl Instance {

    /// Creates a new instance object with the specified application name.
    /// The name of the application is backend specific and may map to different things.
    pub fn create(application_name: & str) -> Instance {
        let gfx_inst = Arc::new(backend::Instance::create(application_name, 1));
        return Instance { gfx_inst };
    }

}

/// The
pub struct Imperium {

    pub instance: Instance,
    pub window: window::Window,
    pub renderer: render::Renderer,

    pub clear_color: Color,

}

impl Imperium {

    pub fn init(application_name: &str) -> Imperium {
        let instance: Instance = Instance::create(application_name);
        let window: window::Window = window::Window::create_fullscreen(application_name).expect("Fatal Error: Failed to create primary window for Imperium Engine.");
        let renderer: render::Renderer = render::Renderer::create(&instance, &window);

        return Imperium { instance, window, renderer, clear_color: Color::black() };
    }

    /// Takes ownership of the current thread and continuously executes the current thread and handles window events.
    /// This function calls the 'update' function to update the component in every iteration of the loop.
    pub fn run(&mut self, component: &mut Component) {

        let mut inst = std::time::Instant::now();

        // Main game loop.
        // TODO: Add window management and events.
        loop {
            let delta: f32 = {
                let dur = inst.elapsed();
                let secs = dur.as_secs() as f32;
                let subsecs = dur.subsec_millis() as f32 / 1000.0;

                secs + subsecs
            };
            inst = std::time::Instant::now();

            if self.update(component, delta) {
                break;
            }
        }

    }

    pub fn update(&mut self, component: &mut Component, delta: f32) -> bool {

        let events: Vec<window::Event> = self.window.collect_events();

        let mut rebuild_swapchain: bool = false;
        let mut quitting: bool = false;

        for event in events.iter() {
            if let window::winit::Event::WindowEvent { event, .. } = event {
                match event {
                    window::winit::WindowEvent::CloseRequested => quitting = true,
                    // We need to recreate our swapchain if we resize, so we'll set
                    // a flag when that happens.
                    window::winit::WindowEvent::Resized(_) => {
                        rebuild_swapchain = true;
                    }
                    _ => {}
                }
            }
        }


        component.handle_events(&events, &mut self.renderer, &mut self.window, delta);

        // Update Cycle.
        // We need to use this scope or else the command dispatch and renderer will be borrowed mutably again later for the render cycle.
        component.update(&mut self.renderer, &mut self.window, delta);

        // Render Cycle.
        if self.renderer.command_dispatch.dispatch_render(self.clear_color, &mut self.renderer.graphics, |graphics, encoder| {
            component.render(graphics, encoder);
        }) {
            rebuild_swapchain = true;
        }
        if quitting {
            return true;
        } else if rebuild_swapchain {
            self.renderer.graphics.render_surface.rebuild(&mut self.renderer.graphics.device, &self.window, &self.renderer.graphics.render_pass, &mut self.renderer.command_dispatch);
        }
        return false;
    }

    pub fn graphics(&mut self) -> &mut render::Graphics {
        return &mut self.renderer.graphics;
    }

}

/// The device structure which contains data about a device.
/// This contains surface data, physical and logical device data as well as graphics queues.
/// This structure must be passed to most graphics objects during initialisation.
pub struct Device {

    pub color_format: gfx::format::Format,
    pub adapter: gfx::Adapter<Backend>,
    pub device: Rc<<Backend as gfx::Backend>::Device>,
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

        let (device, queue_group) =
            adapter.open_with::<_, gfx::Graphics>(1, |family| window_surface.surface.supports_queue_family(family))
                .expect("Fatal Error: Failed to find valid device.");

        // We want to get the capabilities (`caps`) of the surface, which tells us what
        // parameters we can use for our swapchain later. We also get a list of supported
        // image formats for our surface.
        let (caps, formats, _) = window_surface.surface.compatibility(&adapter.physical_device);

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

        return Device { color_format, adapter: adapter, device: Rc::new(device), queue_group, capabilites: caps };

    }

    pub fn load_shader(&self, path: &str) -> Result<<Backend as gfx::Backend>::ShaderModule, &'static str> {

        if let Ok(mut f) = std::fs::File::open(path) {
            let mut contents = String::new();
            if let Ok(_) = f.read_to_string(&mut contents) {
                return Ok(self.device.create_shader_module(&contents.into_bytes()).unwrap());
            } else {
                return Err("Failed to read shader file.");
            }
        } else {
            return Err("Failed to open shader file. Does it exist?");
        }

    }

    pub fn load_shader_raw(&self, bytes: &[u8]) -> Result<<Backend as gfx::Backend>::ShaderModule, &'static str> {
        if let Ok(module) = self.device.create_shader_module(bytes) {
            return Ok(module);
        } else {
            return Err("Failed to create shader module.");
        }
    }

    pub fn upload_type_for(&self, unbound_buffer: &<Backend as gfx::Backend>::UnboundBuffer, properties: gfx::memory::Properties) -> (gfx::MemoryTypeId, gfx::memory::Requirements) {
        let memory_types = self.adapter.physical_device.memory_properties().memory_types;

        let req = self.device.get_buffer_requirements(unbound_buffer);

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

    pub device: Rc<<Backend as gfx::Backend>::Device>,

}

impl DeviceToken {

    /// Creates a device token from a device.
    pub fn create(device: &Device) -> Self {
        return Self { device: device.device.clone() };
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
