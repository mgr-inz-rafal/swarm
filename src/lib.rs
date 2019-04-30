#[macro_export]
macro_rules! bronco {
    ($x: expr, $y: expr) => {
        Bronco {
            pos: gauchos::Position { x: $x, y: $y },
        }
    };
}

#[macro_export]
macro_rules! slot {
    ($x: expr, $y: expr) => {
        Slot {
            pos: gauchos::Position { x: $x, y: $y },
        }
    };
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone)]
pub struct Bronco {
    pub pos: Position,
}

#[derive(Copy, Clone)]
pub struct Slot {
    pub pos: Position,
}

pub struct Gauchos {
    pub broncos: Vec<Bronco>,
    pub slots: Vec<Slot>,
}

pub fn new() -> Gauchos {
    Gauchos::new()
}

impl Gauchos {
    fn new() -> Gauchos {
        Gauchos {
            broncos: Vec::new(),
            slots: Vec::new(),
        }
    }

    fn add_object<T>(vec: &mut Vec<T>, obj: T) {
        vec.push(obj);
    }

    pub fn add_bronco(&mut self, bronco: Bronco) {
        Gauchos::add_object(&mut self.broncos, bronco);
    }

    pub fn get_broncos(&self) -> &Vec<Bronco> {
        &self.broncos
    }

    pub fn add_slot(&mut self, slot: Slot) {
        Gauchos::add_object(&mut self.slots, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broncos_count_zero() {
        assert_eq!(0, 0);
    }
}
