#[macro_use]
extern crate lazy_static;
extern crate itertools;

use itertools::Itertools;
use std::sync::Mutex;

const MAX_GAUCHOS: usize = 8;
const MAX_SLOTS: usize = 16;

#[derive(Copy, Clone)]
struct Gaucho {
    active: bool,
    x: f64,
    y: f64,
    id: usize,
}
#[derive(Copy, Clone)]
struct Slot {
    active: bool,
    x: f64,
    y: f64,
}

struct World {
    gauchos: [Gaucho; MAX_GAUCHOS],
    slots: [Slot; MAX_SLOTS],
}

macro_rules! clear_objects {
    ( $e:expr ) => {
        $e.iter_mut().for_each(|mut x| x.active = false);
    };
}

impl World {
    fn reset(&mut self) {
        clear_objects!(self.gauchos);
        clear_objects!(self.slots);
    }
}

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(World {
        gauchos: [Gaucho {
            active: false,
            x: 0.0,
            y: 0.0,
            id: 666
        }; MAX_GAUCHOS],
        slots: [Slot {
            active: false,
            x: 0.0,
            y: 0.0,
        }; MAX_SLOTS],
    });
}

fn count_gauchos() -> usize {
    let mut counter = 0;
    WORLD.lock().unwrap().gauchos.iter().for_each(|&x| {
        if x.active {
            counter += 1;
        }
    });
    counter
}

macro_rules! insert_object {
    ( $e:expr ) => {{
        match $e.iter().position(|&x| x.active == false) {
            None => Err("No more slots"),
            Some(index) => {
                $e[index].active = true;
                Ok(index)
            }
        }
    }};
}

pub fn add_gaucho() -> Result<usize, &'static str> {
    let mut world = WORLD.lock().unwrap();
    insert_object!(world.gauchos)
}

pub fn add_slot() -> Result<usize, &'static str> {
    let mut world = WORLD.lock().unwrap();
    insert_object!(world.slots)
}

pub fn get_active_gauchos_indices() -> Vec<usize> {
    let mut ret = Vec::new();
    WORLD
        .lock()
        .unwrap()
        .gauchos
        .iter()
        .positions(|x| x.active == true)
        .for_each(|x| ret.push(x));
    ret
}

pub fn get_gaucho_position(index: usize) -> Result<[f64; 2], &'static str> {
    if index >= MAX_GAUCHOS {
        return Err("Index out of bounds");
    }
    let g = WORLD.lock().unwrap().gauchos[index];
    if !g.active {
        Err("No gaucho with requested index")
    } else {
        Ok([g.x, g.y])
    }
}

pub fn set_gaucho_position(index: usize, pos: [f64; 2]) -> Result<(), &'static str> {
    if index >= MAX_GAUCHOS {
        return Err("Index out of bounds");
    }
    let g = &mut WORLD.lock().unwrap().gauchos[index];
    if !g.active {
        Err("No gaucho with requested index")
    } else {
        g.x = pos[0];
        g.y = pos[1];
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauchos_count_zero() {
        WORLD.lock().unwrap().reset();
        assert_eq!(count_gauchos(), 0);
    }

    #[test]
    fn add_gauchos() {
        WORLD.lock().unwrap().reset();
        let _ = add_gaucho();
        let _ = add_gaucho();
        assert_eq!(count_gauchos(), 2);
    }

    #[test]
    fn gaucho_position() {
        WORLD.lock().unwrap().reset();
        let i = add_gaucho();
        let _ = set_gaucho_position(i.unwrap(), [123.0, 70.0]);
        let pos = get_gaucho_position(i.unwrap()).unwrap();
        assert!(pos[0] > 122.0 && pos[1] > 69.0 && pos[0] < 124.0 && pos[1] < 71.0);
    }

    #[test]
    fn count_active_gauchos() {
        WORLD.lock().unwrap().reset();
        let _ = add_gaucho();
        let _ = add_gaucho();
        let _ = add_gaucho();
        assert_eq!(get_active_gauchos_indices().len(), 3);
    }
}
