#[derive(Copy, Clone)]
pub struct Position {
    x: f64,
    y: f64,
}

#[derive(Copy, Clone)]
pub struct Gaucho {
    pos: Position,
}

#[derive(Copy, Clone)]
pub struct Slot {
    position: Position,
}

pub struct World {
    gauchos: Vec<Gaucho>,
    slots: Vec<Slot>,
}

pub fn new_gauchos() -> World {
    World::new()
}

impl World {
    fn new() -> World {
        World {
            gauchos: Vec::new(),
            slots: Vec::new(),
        }
    }

    fn add_object<T>(vec: &mut Vec<T>, obj: T) {
        vec.push(obj);
    }

    pub fn add_gaucho(&mut self, gaucho: Gaucho) {
        World::add_object(&mut self.gauchos, gaucho);
    }

    pub fn add_slot(&mut self, slot: Slot) {
        World::add_object(&mut self.slots, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauchos_count_zero() {
        assert_eq!(0, 0);
    }
}
