use crate::*;

use std::time;

pub trait AppController {

    fn update(&mut self, cycle: &mut UpdateCycle);

    fn handle_events(&mut self, events: &Vec<window::Event>, cycle: &mut UpdateCycle) {}

}

pub struct UpdateCycle<'a> {
    pub interface: &'a mut AppInterface,
    pub next_controller: Option<Box<AppController>>,
    pub delta: f32,
}

impl<'a> UpdateCycle<'a> {
    pub fn new(interface: &'a mut AppInterface, delta: f32) -> Self {
        return Self { interface, next_controller: None, delta };
    }
}

pub struct AppInterface {
    pub instance: core::Instance,
    pub window: window::Window,
    pub graphics: render::Graphics,
    pub clear_color: Color,
    pub should_terminate: bool,
}

impl AppInterface {

    pub fn new(application_name: &str) -> Self {
        let instance: core::Instance = core::Instance::create(application_name);
        let window: window::Window = window::Window::create_fullscreen(application_name).expect("Fatal Error: Failed to create primary window for Instance Engine.");
        let graphics: render::Graphics = render::Graphics::create(&instance, &window);

        return Self { instance, window, graphics, clear_color: Color::black(), should_terminate: false };
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
        if !self.graphics.render_surface.is_valid {
            self.graphics.render_surface.rebuild(&self.window, &mut self.graphics.device);
            log!(debug, 0, "Rebuilding swapchain.");
        } else {
            if self.graphics.render_surface.did_rebuild {
                self.graphics.render_surface.did_rebuild = false;
            }
        }
    }

    pub fn invalidate_surface(&mut self) {
        self.graphics.render_surface.invalidate();
    }
}

pub enum LoopInstruction {

    Continue,
    Exit,

}

pub struct App {

    pub interface: AppInterface,
    pub controller: Box<AppController>,

    prev_time: time::Instant,

}

impl App {

    pub fn new(interface: AppInterface, controller: Box<AppController>) -> Self {
        return Self { interface, controller, prev_time: time::Instant::now() };
    }

    pub fn update(&mut self) -> LoopInstruction {
        let delta: f32 = {
            let delta_dur = self.prev_time.elapsed();
            self.prev_time = time::Instant::now();
            delta_dur.as_float_secs() as f32
        };
        {
            let events: Vec<window::Event> = self.interface.poll_events();
            let mut cycle: UpdateCycle = UpdateCycle::new(&mut self.interface, delta);
            self.controller.handle_events(&events, &mut cycle);
            self.controller.update(&mut cycle);
        }
        if self.interface.should_terminate {
            return LoopInstruction::Exit;
        }
        self.interface.update();
        return LoopInstruction::Continue;
    }
}