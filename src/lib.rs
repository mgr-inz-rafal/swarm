#[macro_use]
extern crate approx;

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

const ANGLE_INCREMENT: f64 = 0.05;
const SPEED_FACTOR: f64 = 2.0;

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
        let carrier = &mut carriers[0];
        match carrier.state {
            State::IDLE => {
                carrier.state = State::TARGETING((200.0, 500.0));
            }
            _ => {}
        }
    }
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

    pub fn tick(&mut self) {
        Dispatcher::conduct(&mut self.carriers);
        self.carriers.iter_mut().for_each(|x| x.tick());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    IDLE,
    TARGETING((f64, f64)),
    MOVING((f64, f64)),
    _DEBUG_,
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

    pub fn get_target(&self) -> Option<(f64, f64)> {
        match self.state {
            State::TARGETING(target) => Some(target),
            State::MOVING(target) => Some(target),
            _ => None,
        }
    }

    fn calculate_angle_to(&self, target: (f64, f64)) -> f64 {
        let mut angle = (target.1 - self.pos.y).atan2(target.0 - self.pos.x);
        if angle < 0.0 {
            angle += std::f64::consts::PI * 2.0;
        }
        angle
    }

    fn rotate_to(&mut self, target_angle: f64) {
        self.angle += ANGLE_INCREMENT;
        if self.angle > std::f64::consts::PI * 2.0 {
            self.angle = self.angle - std::f64::consts::PI * 2.0;
        }
    }

    fn move_forward(&mut self) {
        self.pos.x = self.pos.x + self.angle.cos() * SPEED_FACTOR;
        self.pos.y = self.pos.y + self.angle.sin() * SPEED_FACTOR;
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
        match self.state {
            State::TARGETING(target) => {
                let target_angle = self.calculate_angle_to(target);

                if !relative_eq!(target_angle, self.angle, epsilon = ANGLE_INCREMENT * 1.2) {
                    self.rotate_to(target_angle)
                } else {
                    self.angle = target_angle;
                    self.state = State::MOVING(target);
                }
            }
            State::MOVING(_) => {
                self.move_forward();
            }
            _ => {}
        }
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
