use crate::framework::World;

pub type SystemType = Box<dyn System + Send>;

pub trait System {
    fn setup(&mut self, world: &mut World);
    fn run(&mut self);
    fn dispose(&mut self);
}
