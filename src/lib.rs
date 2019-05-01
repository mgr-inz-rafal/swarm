pub mod carrier;
pub mod tools;

use carrier::Carrier;
use tools::Position;

#[macro_export]
macro_rules! carrier {
    ($x: expr, $y: expr) => {
        Carrier {
            pos: swarm::tools::Position { x: $x, y: $y },
        }
    };
}

#[macro_export]
macro_rules! slot {
    ($x: expr, $y: expr) => {
        Slot {
            pos: swarm::tools::Position { x: $x, y: $y },
        }
    };
}

#[derive(Copy, Clone)]
pub struct Slot {
    pub pos: Position,
}

pub struct Swarm {
    carriers: Vec<Carrier>,
    slots: Vec<Slot>,
}

pub fn new() -> Swarm {
    Swarm::new()
}

impl Swarm {
    fn new() -> Swarm {
        Swarm {
            carriers: Vec::new(),
            slots: Vec::new(),
        }
    }

    fn add_object<T>(vec: &mut Vec<T>, obj: T) {
        vec.push(obj);
    }

    pub fn add_carrier(&mut self, carrier: Carrier) {
        Swarm::add_object(&mut self.carriers, carrier);
    }

    pub fn get_carriers(&self) -> &Vec<Carrier> {
        &self.carriers
    }

    pub fn get_slots(&self) -> &Vec<Slot> {
        &self.slots
    }

    pub fn add_slot(&mut self, slot: Slot) {
        Swarm::add_object(&mut self.slots, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carriers_count_zero() {
        assert_eq!(0, 0);
    }
}
