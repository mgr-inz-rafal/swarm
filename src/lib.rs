#[macro_use]
extern crate itertools;

use itertools::Itertools;
use std::sync::Mutex;

const MAX_GAUCHOS: usize = 8;
const MAX_SLOTS: usize = 16;

pub trait Positionable {
    fn get_coords(&self) -> (f64, f64);
    fn set_coords(&mut self, position: (f64, f64));
}

#[derive(Copy, Clone)]
pub struct Position {
    x: f64,
    y: f64,
}
impl Positionable for Position {
    fn get_coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    fn set_coords(&mut self, position: (f64, f64)) {
        self.x = position.0;
        self.y = position.1;
    }
}

pub trait HasPosition {
    fn get_position(&self) -> &Position;
    fn get_position_mut(&mut self) -> &mut Position;
}
macro_rules! impl_HasPosition {
    ( for $($type:ty),+ ) => {
        $(impl HasPosition for $type {
            fn get_position_mut(&mut self) -> &mut Position {
                &mut self.position
            }
            fn get_position(&self) -> &Position {
                &self.position
            }
        })*
    };
}

pub trait Activable {
    fn is_active(&self) -> bool;
    fn activate(&mut self);
    fn deactivate(&mut self);
}

#[derive(Copy, Clone)]
pub struct Activator {
    active: bool,
}
impl Activable for Activator {
    fn is_active(&self) -> bool {
        self.active
    }
    fn activate(&mut self) {
        self.active = true;
    }
    fn deactivate(&mut self) {
        self.active = false;
    }
}

pub trait HasActivator {
    fn get_activator(&self) -> &Activator;
    fn get_activator_mut(&mut self) -> &mut Activator;
}
macro_rules! impl_HasActivator {
    ( for $($type:ty),+ ) => {
        $(impl HasActivator for $type {
            fn get_activator_mut(&mut self) -> &mut Activator {
                &mut self.activator
            }
            fn get_activator(&self) -> &Activator {
                &self.activator
            }
        })*
    };
}

#[derive(Copy, Clone)]
struct Gaucho {
    activator: Activator,
    position: Position,
    id: usize,
}
#[derive(Copy, Clone)]
struct Slot {
    activator: Activator,
    position: Position,
}
impl_HasActivator!(for Gaucho, Slot);
impl_HasPosition!(for Gaucho, Slot);

pub struct World {
    gauchos: [Gaucho; MAX_GAUCHOS],
    slots: [Slot; MAX_SLOTS],
}

pub fn new_gauchos() -> World {
    World::new()
}

impl World {
    fn new() -> World {
        World {
            gauchos: [Gaucho {
                activator: Activator { active: false },
                position: Position { x: 0.0, y: 0.0 },
                id: 666,
            }; MAX_GAUCHOS],
            slots: [Slot {
                activator: Activator { active: false },
                position: Position { x: 0.0, y: 0.0 },
            }; MAX_SLOTS],
        }
    }

    fn add_object<T: HasActivator>(arr: &mut [T]) -> Result<usize, &'static str> {
        match arr.iter().position(|x| !x.get_activator().is_active()) {
            None => Err("No more slots"),
            Some(index) => {
                arr[index].get_activator_mut().activate();
                Ok(index)
            }
        }
    }

    fn count_active_objects<T: HasActivator>(arr: &[T]) -> usize {
        let mut counter = 0;
        arr.iter().for_each(|x| {
            if x.get_activator().is_active() {
                counter += 1;
            }
        });
        counter
    }

    fn set_object_position<T: HasPosition>(obj: &mut T, position: (f64, f64)) {
        obj.get_position_mut().set_coords(position);
    }

    fn get_object_position<T: HasPosition>(obj: &mut T) -> (f64, f64) {
        obj.get_position().get_coords()
    }

    pub fn add_gaucho(&mut self) -> Result<usize, &'static str> {
        World::add_object(&mut self.gauchos)
    }

    pub fn add_slot(&mut self) -> Result<usize, &'static str> {
        World::add_object(&mut self.slots)
    }

    pub fn count_gauchos(&self) -> usize {
        World::count_active_objects(&self.gauchos)
    }

    pub fn count_slots(&self) -> usize {
        World::count_active_objects(&self.slots)
    }

    pub fn set_gaucho_position(
        &mut self,
        index: usize,
        position: (f64, f64),
    ) -> Result<(), &'static str> {
        // TODO: Copy & pasted prelude to function
        if index >= MAX_GAUCHOS {
            return Err("Index out of bounds");
        }
        if !self.gauchos[index].get_activator().is_active() {
            return Err("No such gaucho");
        }

        World::set_object_position(&mut self.gauchos[index], position);
        Ok(())
    }

    pub fn get_gaucho_position(&mut self, index: usize) -> Result<(f64, f64), &'static str> {
        // TODO: Copy & pasted prelude to function
        if index >= MAX_GAUCHOS {
            return Err("Index out of bounds");
        }
        if !self.gauchos[index].get_activator().is_active() {
            return Err("No such gaucho");
        }

        Ok(World::get_object_position(&mut self.gauchos[index]))
    }

    pub fn set_slot_position(
        &mut self,
        index: usize,
        position: (f64, f64),
    ) -> Result<(), &'static str> {
        // TODO: Copy & pasted prelude to function
        if index >= MAX_SLOTS {
            return Err("Index out of bounds");
        }
        if !self.slots[index].get_activator().is_active() {
            return Err("No such slot");
        }

        World::set_object_position(&mut self.slots[index], position);
        Ok(())
    }

    pub fn get_slot_position(&mut self, index: usize) -> Result<(f64, f64), &'static str> {
        // TODO: Copy & pasted prelude to function
        if index >= MAX_SLOTS {
            return Err("Index out of bounds");
        }
        if !self.slots[index].get_activator().is_active() {
            return Err("No such slot");
        }

        Ok(World::get_object_position(&mut self.slots[index]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauchos_count_zero() {
        let world = new_gauchos();
        assert_eq!(world.count_gauchos(), 0);
    }

    #[test]
    fn add_gauchos() {
        let mut world = new_gauchos();
        let _ = world.add_gaucho();
        let _ = world.add_gaucho();
        assert_eq!(world.count_gauchos(), 2);
    }

    #[test]
    fn add_slots() {
        let mut world = new_gauchos();
        let _ = world.add_slot();
        let _ = world.add_slot();
        let _ = world.add_slot();
        let _ = world.add_slot();
        assert_eq!(world.count_slots(), 4);
    }

    #[test]
    fn gaucho_position() {
        let mut world = new_gauchos();
        let i = world.add_gaucho();
        let _ = world.set_gaucho_position(i.unwrap(), (123.0, 70.0));
        let pos = world.get_gaucho_position(i.unwrap()).unwrap();
        assert!(pos.0 > 122.0 && pos.1 > 69.0 && pos.0 < 124.0 && pos.1 < 71.0);
    }

    #[test]
    fn slot_position() {
        let mut world = new_gauchos();
        let i = world.add_slot();
        let _ = world.set_slot_position(i.unwrap(), (50.0, 150.0));
        let pos = world.get_slot_position(i.unwrap()).unwrap();
        assert!(pos.0 > 49.0 && pos.1 > 51.0 && pos.0 < 149.0 && pos.1 < 151.0);
    }
}
