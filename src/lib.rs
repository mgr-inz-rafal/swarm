#[macro_export]
macro_rules! carrier {
    ($x: expr, $y: expr) => {
        Carrier::new($x, $y)
    };
}

#[macro_export]
macro_rules! slot {
    ($x: expr, $y: expr) => {
        Slot::new($x, $y)
    };
}

#[derive(Copy, Clone)]
pub struct Slot {
    pos: Position,
}
impl Slot {
    pub fn new(x: f64, y: f64) -> Slot {
        Slot {
            pos: Position::new(x, y),
        }
    }
    pub fn get_position(&self) -> &Position {
        &self.pos
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn conduct(carriers: &mut Vec<Carrier>) {
        let mut carrier = carriers[0];
        match carrier.state {
            _ => {}
            State::IDLE => {
                carrier.state = State::TARGETING((300.0, 300.0));
            }
        }
    }
}

pub struct Swarm {
    carriers: Vec<Carrier>,
    slots: Vec<Slot>,
    dispatcher: Dispatcher,
}

pub fn new() -> Swarm {
    Swarm::new()
}

impl Swarm {
    fn new() -> Swarm {
        Swarm {
            carriers: Vec::new(),
            slots: Vec::new(),
            dispatcher: Dispatcher {},
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

    pub fn tick(&mut self) {
        Dispatcher::conduct(&mut self.carriers);
        self.carriers.iter_mut().for_each(|x| x.tick());
    }
}

#[derive(Copy, Clone)]
pub enum State {
    IDLE,
    TARGETING((f64, f64)),
    MOVING,
}

#[derive(Copy, Clone)]
pub struct Carrier {
    pos: Position,
    angle: f64,
    state: State,
}

impl Carrier {
    pub fn new(x: f64, y: f64) -> Carrier {
        Carrier {
            pos: Position::new(x, y),
            angle: 0.0,
            state: State::IDLE,
        }
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn tick(&mut self) {
        /*
        self.pos.x = self.pos.x + self.angle.cos();
        self.pos.y = self.pos.y + self.angle.sin();
        self.angle += 0.01
        */
    }
    
}

#[derive(Copy, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Position {
        Position { x, y }
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
