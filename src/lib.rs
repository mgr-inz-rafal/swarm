#[macro_use]
extern crate itertools;

use itertools::Itertools;
use std::sync::Mutex;

const MAX_GAUCHOS: usize = 8;
const MAX_SLOTS: usize = 16;

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
    x: f64,
    y: f64,
    id: usize,
}
#[derive(Copy, Clone)]
struct Slot {
    activator: Activator,
    x: f64,
    y: f64,
}
impl_HasActivator!(for Gaucho, Slot);

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
                x: 0.0,
                y: 0.0,
                id: 666,
            }; MAX_GAUCHOS],
            slots: [Slot {
                activator: Activator { active: false },
                x: 0.0,
                y: 0.0,
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

    /*
    fn set_object_position
    */

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

    /*
    pub fn set_gaucho_position(&self, index: usize, pos: [f64; 2]) -> Result<(), &'static str> {
        if index >= MAX_GAUCHOS {
            Err(format!("Max index for Gaucho is {}", MAX_GAUCHOS));
        }
        set_object_position(self.gauchos[index], pos)
    }
    */
}

/*
fn get_active_objects_indices<T: HasActivator>(arr: &[T]) -> Vec<usize> {
    let mut ret = Vec::new();
    arr.iter()
        .positions(|x| x.get_activator().is_active())
        .for_each(|x| ret.push(x));
    ret
}

pub fn get_active_gauchos_indices(world: &World) -> Vec<usize> {
    let gauchos = world.gauchos;
    get_active_objects_indices(&gauchos)
}
*/

/*
pub fn get_active_slots_indices() -> Vec<usize> {
    let slots = &WORLD.lock().unwrap().slots;
    get_active_objects_indices(slots)
}
*/

/*
macro_rules! get_object_position {
    ( $e:expr, $i_index: ident, $i_max: ident ) => {{
        if $i_index >= $i_max {
            return Err("Index out of bounds");
        }
        let obj = $e[$i_index];
        if !obj.get_activator().is_active() {
            Err("No object with requested index")
        } else {
            Ok([obj.x, obj.y])
        }
    }};
}

pub fn get_gaucho_position(index: usize) -> Result<[f64; 2], &'static str> {
    let g = WORLD.lock().unwrap().gauchos;
    get_object_position!(g, index, MAX_GAUCHOS)
}

pub fn get_slot_position(index: usize) -> Result<[f64; 2], &'static str> {
    let s = WORLD.lock().unwrap().slots;
    get_object_position!(s, index, MAX_GAUCHOS)
}
*/

/*
macro_rules! set_object_position {
    ( $e:expr, $i_index: ident, $i_pos: ident, $i_max: ident ) => {{
        if $i_index >= $i_max {
            return Err("Index out of bounds");
        }
        let obj = &mut $e[$i_index];
        if !obj.get_activator().is_active() {
            Err("No object with requested index")
        } else {
            obj.x = $i_pos[0];
            obj.y = $i_pos[1];
            Ok(())
        }
    }};
}

pub fn set_gaucho_position(index: usize, pos: [f64; 2]) -> Result<(), &'static str> {
    let world = &mut WORLD.lock().unwrap();
    set_object_position!(world.gauchos, index, pos, MAX_GAUCHOS)
}

pub fn set_slot_position(index: usize, pos: [f64; 2]) -> Result<(), &'static str> {
    let world = &mut WORLD.lock().unwrap();
    set_object_position!(world.slots, index, pos, MAX_SLOTS)
}
*/

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

    /*
    #[test]
    fn gaucho_position() {
        let mut world = new_gauchos();
        let i = world.add_gaucho();
        let _ = world.set_gaucho_position(i.unwrap(), [123.0, 70.0]);
        let pos = world.get_gaucho_position(i.unwrap()).unwrap();
        assert!(pos[0] > 122.0 && pos[1] > 69.0 && pos[0] < 124.0 && pos[1] < 71.0);
    }
    */

    /*

    #[test]
    fn slot_position() {
        WORLD.lock().unwrap().reset();
        let i = add_slot();
        let _ = set_slot_position(i.unwrap(), [50.0, 150.0]);
        let pos = get_slot_position(i.unwrap()).unwrap();
        assert!(pos[0] > 49.0 && pos[1] > 51.0 && pos[0] < 149.0 && pos[1] < 151.0);
    }

    #[test]
    fn count_active_gauchos() {
        WORLD.lock().unwrap().reset();
        let _ = add_gaucho();
        let _ = add_gaucho();
        let _ = add_gaucho();
        assert_eq!(get_active_gauchos_indices().len(), 3);
    }

    #[test]
    fn count_active_slots() {
        WORLD.lock().unwrap().reset();
        let _ = add_slot();
        let _ = add_slot();
        let _ = add_slot();
        let _ = add_slot();
        assert_eq!(get_active_slots_indices().len(), 4);
    }
    */
}
