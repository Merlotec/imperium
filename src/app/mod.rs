use crate::*;

use std::time;

pub trait View {

    fn update(&mut self, cycle: &mut UpdateCycle);

    fn handle_events(&mut self, events: &Vec<window::Event>, cycle: &mut UpdateCycle) {}

    fn render(&mut self, graphics: &mut render::Graphics, encoder: &mut command::Encoder);

}

pub struct UpdateCycle<'a> {
    pub interface: &'a mut AppInterface,
    pub next_view: Option<Box<View>>,
    pub delta: f32,
}

impl<'a> UpdateCycle<'a> {
    pub fn new(interface: &'a mut AppInterface, delta: f32) -> Self {
        return Self { interface, next_view: None, delta };
    }
}

pub struct AppInterface {
    pub instance: core::Instance,
    pub window: window::Window,
    pub renderer: render::Renderer,
    pub clear_color: Color,
    pub should_terminate: bool,
}

impl AppInterface {

    pub fn new(application_name: &str) -> Self {
        let instance: core::Instance = core::Instance::create(application_name);
        let window: window::Window = window::Window::create_fullscreen(application_name).expect("Fatal Error: Failed to create primary window for Instance Engine.");
        let renderer: render::Renderer = render::Renderer::create(&instance, &window);

        return Self { instance, window, renderer, clear_color: Color::black(), should_terminate: false };
    }

    pub fn poll_events(&mut self) -> Vec<window::Event> {
        let events: Vec<window::Event> = self.window.collect_events();
        for event in events.iter() {
            if let window::winit::Event::WindowEvent { event, .. } = event {
                match event {
                    window::winit::WindowEvent::CloseRequested => self.should_terminate = true,
                    // We need to recreate our swapchain if we resize, so we'll set
                    // a flag when that happens.
                    window::winit::WindowEvent::Resized(_) => {
                        self.invalidate_surface();
                    }
                    _ => {}
                }
            }
        }
        return events;
    }

    pub fn update(&mut self) {
        if !self.renderer.graphics.render_surface.is_valid {
            self.renderer.graphics.render_surface.rebuild(&mut self.renderer.graphics.device, &self.window, &self.renderer.graphics.render_pass, &mut self.renderer.command_dispatch);
            log!(debug, 0, "Rebuilding swapchain.");
        }
    }

    pub fn invalidate_surface(&mut self) {
        self.renderer.graphics.render_surface.invalidate();
    }

    pub fn graphics(&mut self) -> &mut render::Graphics {
        return &mut self.renderer.graphics;
    }
}

pub struct App {

    pub interface: AppInterface,
    pub view: Box<View>,

    prev_time: time::Instant,

}

impl App {

    pub fn new(interface: AppInterface, view: Box<View>) -> Self {
        return Self { interface, view, prev_time: time::Instant::now() };
    }

    pub fn exec_frame(&mut self) {
        let delta: f32 = {
            let delta_dur = self.prev_time.elapsed();
            self.prev_time = time::Instant::now();
            delta_dur.as_float_secs() as f32
        };
        {
            let events: Vec<window::Event> = self.interface.poll_events();
            let mut cycle: UpdateCycle = UpdateCycle::new(&mut self.interface, delta);
            self.view.handle_events(&events, &mut cycle);
            self.view.update(&mut cycle);
        }
        {
            let view: &mut View = self.view.as_mut();
            if !self.interface.renderer.render(self.interface.clear_color, |graphics, encoder| {
                view.render(graphics, encoder);
            }) {
                self.interface.renderer.graphics.render_surface.invalidate();
            }
        }

        if !self.interface.renderer.graphics.render_surface.is_valid {
            self.interface.renderer.graphics.render_surface.rebuild(
                &mut self.interface.renderer.graphics.device,
                &mut self.interface.window,
                &mut self.interface.renderer.graphics.render_pass,
                &mut self.interface.renderer.command_dispatch
            );
            log!(debug, 0, "Rebuilding swapchain!");
        } else if self.interface.renderer.graphics.render_surface.did_rebuild {
            self.interface.renderer.graphics.render_surface.did_rebuild = false;
        }
    }
}