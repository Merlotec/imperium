use crate::*;

#[derive(Copy, Clone)]
pub enum Trigger {
    KeyTrigger(window::winit::VirtualKeyCode, window::winit::ElementState),
    MouseButtonTrigger(window::winit::MouseButton, window::winit::ElementState),
}

#[derive(Copy, Clone)]
pub enum TriggerType {
    Once,
    Toggle,
}

#[derive(Clone)]
pub struct MovementState {

    pub id: String,
    pub trigger: Trigger,
    pub trigger_type: TriggerType,

    is_activated: bool,
    is_triggered: bool,

}

impl MovementState {
    pub fn new(id: String, trigger: Trigger, trigger_type: TriggerType) -> Self {
        return Self { id, trigger, trigger_type, is_activated: false, is_triggered: false };
    }
    pub fn set_active(&mut self, b: bool) {
        self.is_activated = b;
        self.is_triggered = b;

    }
    pub fn tick(&mut self) {
        if let TriggerType::Once = self.trigger_type {
            if !self.is_activated {
                self.is_triggered = false;
            }
        }
        self.is_activated = false;
    }
    pub fn triggered(&self) -> bool {
        return self.is_triggered;
    }
}

pub struct MovementStateHandler {

    pub states: Vec<MovementState>,
    pub cursor_pos: Vector2f,

}

impl MovementStateHandler {

    pub fn new() -> Self {
        return Self { states: Vec::new(), cursor_pos: Vector2f::zero() };
    }

    pub fn add_state(&mut self, state: MovementState) {
        self.states.push(state);
    }

    pub fn is_triggered(&self, id: String) -> Result<bool, &'static str> {
        for state in self.states.iter() {
            if state.id == id {
                return Ok(state.triggered());
            }
        }
        return Err("Failed to find state with the specified id.");
    }

    pub fn handle_events(&mut self, events: &Vec<window::Event>) {
        for event in events.iter() {
            if let window::winit::Event::WindowEvent { event, .. } = event {
                match event {
                    window::winit::WindowEvent::KeyboardInput {
                        input:
                        window::winit::KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        },
                        ..
                    } => {
                        for movement_state in self.states.iter_mut() {
                            if let Trigger::KeyTrigger(key_code, key_state) = movement_state.trigger {
                                if let Some(vkc) = virtual_keycode {
                                    if *vkc == key_code {
                                        if *state == key_state {
                                            movement_state.set_active(true);
                                        } else {
                                            movement_state.set_active(false);
                                        }
                                    }
                                }
                            }
                        }
                    },
                    window::winit::WindowEvent::MouseInput {
                        state,
                        button,
                        ..
                    } => {
                        for movement_state in self.states.iter_mut() {
                            if let Trigger::MouseButtonTrigger(mouse_button, mouse_state) = movement_state.trigger {
                                if *button == mouse_button {
                                    if *state == mouse_state {
                                        movement_state.set_active(true);
                                    } else {
                                        movement_state.set_active(false);
                                    }
                                }
                            }
                        }
                    },
                    window::winit::WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        self.cursor_pos = Vector2f::new(position.x as f32, position.y as f32);
                    },
                    _ => {}
                }
            }
        }
        for state in self.states.iter_mut() {
            state.tick();
        }
    }

}