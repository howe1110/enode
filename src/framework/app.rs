use crate::framework::{SystemType, World};
use std::io::{self};

pub struct Application {
    systems: Vec<SystemType>,
    world: World,
}

impl Application {
    pub fn new() -> Self {
        let systems = Vec::new();
        let world = World::empty();
        Application { systems, world }
    }
    /// Sets up the application.
    fn initialize(&mut self) {}
    pub fn run(&mut self) {
        
        self.initialize();
        for sys in self.systems.iter_mut() {
            sys.setup(&mut self.world);
            sys.run();
        }

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        println!("{}", input);

        self.shutdown();
    }
    /// Cleans up after the quit signal is received.
    fn shutdown(&mut self) {}
    pub fn with(mut self, system: SystemType) -> Self {
        self.systems.push(system);
        self
    }
}
