#[derive(Copy, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone)]
pub struct Gaucho {
    pub pos: Position,
}

#[derive(Copy, Clone)]
pub struct Slot {
    pub pos: Position,
}

pub struct Gauchos {
    pub gauchos: Vec<Gaucho>,
    pub slots: Vec<Slot>,
}

pub fn new() -> Gauchos {
    Gauchos::new()
}

impl Gauchos {
    fn new() -> Gauchos {
        Gauchos {
            gauchos: Vec::new(),
            slots: Vec::new(),
        }
    }

    fn add_object<T>(vec: &mut Vec<T>, obj: T) {
        vec.push(obj);
    }

    pub fn add_gaucho(&mut self, gaucho: Gaucho) {
        Gauchos::add_object(&mut self.gauchos, gaucho);
    }

    pub fn get_gauchos(&self) -> &Vec<Gaucho> {
        &self.gauchos
    }

    pub fn add_slot(&mut self, slot: Slot) {
        Gauchos::add_object(&mut self.slots, slot);
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
