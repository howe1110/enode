use std::any::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct World {
    resources: HashMap<TypeId, Box<dyn Any>>,
}


impl World {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn insert<R>(&mut self, r: R)
    where
        R: Any,
    {
        self.resources.insert(TypeId::of::<R>(), Box::new(r));
    }

    pub fn remove<R>(&mut self) -> Option<R>
    where
        R: Any,
    {
        self.resources
            .remove(&TypeId::of::<R>())
            .map(|x: Box<dyn Any>| *(x.downcast::<R>().unwrap()))
    }
}