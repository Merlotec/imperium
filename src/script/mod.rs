use crate::*;
use std::collections::HashMap;
use std::thread;

pub trait Script<A: scene::Aggregator> {

    fn init() -> Self where Self : Sized;

    /// Returns exit code.
    fn run(&mut self, scene: &mut scene::Scene<A>) -> u32;

}

pub struct ScriptController<A: scene::Aggregator> {

    pub scripts: HashMap<String, Box<Script<A>>>,

}

impl<A: scene::Aggregator> ScriptController<A> {

    pub fn new() -> Self {
        return Self { scripts: HashMap::new() };
    }

    pub fn register_script(&mut self, script_id: String, script: Box<Script<A>>) {
        self.scripts.insert(script_id, script);
    }

    /// Returns the exit code of the invoked script.
    pub fn invoke(&mut self, script_id: &str, scene: &mut scene::Scene<A>) -> Result<u32, &'static str> {
        if let Some(script) = self.scripts.get_mut(script_id) {
            return Ok(script.run(scene));
        }
        return Err("Failed to invoke the script - no script found with the specified id.");
    }

}