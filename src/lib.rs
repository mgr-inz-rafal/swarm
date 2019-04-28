#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

const MAX_GAUCHOS: usize = 16;

#[derive(Copy, Clone)]
struct Gaucho {
    active: bool,
    x: f64,
    y: f64,
}

struct World {
    gauchos: [Gaucho; MAX_GAUCHOS],
}

impl World {
    fn reset(&mut self) {
        self.clear_gauchos();
    }
    fn clear_gauchos(&mut self) {
        self.gauchos.iter_mut().for_each(|mut x| x.active = false);
    }
}

lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(World {
        gauchos: [Gaucho {
            active: false,
            x: 0.0,
            y: 0.0,
        }; MAX_GAUCHOS],
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

fn find_exmpty_slot() -> Option<usize> {
    WORLD
        .lock()
        .unwrap()
        .gauchos
        .iter()
        .position(|&x| x.active == false)
}

fn add_gaucho() -> Result<(), &'static str> {
    match find_exmpty_slot() {
        None => Err("No more slots"),
        Some(index) => {
            WORLD.lock().unwrap().gauchos[index].active = true;
            Ok(())
        }
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
}
