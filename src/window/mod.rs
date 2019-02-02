use super::*;

pub use crate::winit;

pub type Event = winit::Event;

pub struct WindowCreationError(&'static str);

/// Represents a window in the Imperium engine.
/// Each window should have its own 'Renderer' which represents the render pipeline targeting a window.
/// The window can be created with any 'create' method (e.g. 'create' or 'create_fullscreen').
/// To create a window of size (800, 500) and with a title of "Test Title" we can use the following code:
/// ````
/// let window: Window = Window::create("Test Title", Vector2f::new(800.0, 500.0));
/// ````
pub struct Window {

    pub handle: winit::Window,
    pub events_loop: winit::EventsLoop,

    pub lock_cursor: bool,

}

impl Window {

    pub fn create(title: &str, size: Vector2f) -> Result<Window, &'static str> {
        let events_loop = winit::EventsLoop::new();
        let builder = winit::WindowBuilder::new().with_title(title).with_dimensions(winit::dpi::LogicalSize::new(size.x as f64, size.y as f64));
        if let Ok(window) = builder.build(&events_loop) {
            return Ok(Window { handle: window, events_loop, lock_cursor: false });
        }
        return Err("Failed to create window (winit).");
    }

    pub fn create_fullscreen(title: &str) -> Result<Window, &'static str> {
        let events_loop = winit::EventsLoop::new();
        let builder = winit::WindowBuilder::new().with_title(title).with_fullscreen(None);
        if let Ok(window) = builder.build(&events_loop) {
            return Ok(Window { handle: window, events_loop, lock_cursor: false });
        }
        return Err("Failed to create window (winit).");
    }

    pub fn update(&mut self) {

        if self.lock_cursor {
            let size = self.handle.get_inner_size().expect("WINDOW ERROR!");
            self.handle.set_cursor_position(winit::dpi::LogicalPosition { x: size.width / 2.0, y: size.height / 2.0 } );
        }

    }

    /// Collects a list of events generated at the last frame.
    pub fn collect_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        self.events_loop.poll_events(|event| {
            events.push(event);
        });
        return events;
    }

    pub fn set_cursor(&mut self, cursor: winit::MouseCursor) {
        self.handle.set_cursor(cursor);
    }

    pub fn get_size(&self) -> Vector2f {

        let size = self.handle.get_inner_size().expect("Failed to get window size!");

        return Vector2f::new(size.width as f32, size.height as f32);

    }

    pub fn get_pos(&self) -> Vector2f {
        let pos = self.handle.get_position().expect("Failed to get window size!");

        return Vector2f::new(pos.x as f32, pos.y as f32);
    }

}

/// This structure represents the graphics surface object of a window.
pub struct WindowSurface {
    pub surface: <Backend as gfx::Backend>::Surface,
    pub size: Vector2f,
}

impl WindowSurface {

    /// Creates a new surface with the specified instance and window.
    pub fn create(instance: &core::Instance, window: &window::Window) -> Self {
        let surface: <Backend as gfx::Backend>::Surface = instance.gfx_inst.create_surface(&window.handle);
        let size = window.get_size();
        return Self { surface, size };
    }

}