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
    ($x: expr, $y: expr, $cp: expr, $tp: expr) => {
        Slot::new($x, $y, $cp, $tp)
    };
}

// TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Payload {
    pub cargo: char,
}

impl Payload {
    pub fn from_char(c: char) -> Payload {
        Payload { cargo: c }
    }
}

const ANGLE_INCREMENT: f64 = 0.05;
const SPEED_FACTOR: f64 = 2.0;
const POSITION_EQUALITY_EPSILON: f64 = SPEED_FACTOR * 1.5;

#[derive(Copy, Clone)]
pub struct Slot {
    pos: Position,
    current_payload: Option<Payload>,
    target_payload: Option<Payload>,
    taken_care_of: bool,
}
impl Slot {
    pub fn new(
        x: f64,
        y: f64,
        current_payload: Option<Payload>,
        target_payload: Option<Payload>,
    ) -> Slot {
        Slot {
            pos: Position::new(x, y),
            current_payload,
            target_payload,
            taken_care_of: false,
        }
    }
    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_payloads(&self) -> [Option<Payload>; 2] {
        [self.current_payload, self.target_payload]
    }

    pub fn is_taken_care_of(&self) -> bool {
        self.taken_care_of
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn conduct(carriers: &mut Vec<Carrier>, slots: &mut Vec<Slot>) {
        carriers.iter_mut().for_each(|mut x| match x.state {
            State::IDLE => {
                if let Some(slot_index) = Dispatcher::find_slot_with_mismatched_payload(slots) {
                    x.target_slot(slot_index, &mut slots[slot_index])
                }
            }
            State::LOOKINGFORTARGET => match Dispatcher::find_slot_for_target(slots, x.payload) {
                Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index]),
                None => x.state = State::NOTARGET,
            },
            State::NOTARGET => match Dispatcher::find_temporary_slot(slots, x.payload) {
                Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index]),
                None => x.state = State::LOOKINGFORTARGET,
            },
            _ => {}
        })
    }

    fn find_slot_with_mismatched_payload(slots: &[Slot]) -> Option<usize> {
        slots.iter().position(|x| {
            x.current_payload != None && x.current_payload != x.target_payload && !x.taken_care_of
        })
    }

    fn find_slot_for_target(slots: &[Slot], target: Option<Payload>) -> Option<usize> {
        slots.iter().position(|x| {
            x.current_payload == None && x.target_payload == target && !x.taken_care_of
        })
    }

    fn find_temporary_slot(slots: &[Slot], target: Option<Payload>) -> Option<usize> {
        slots
            .iter()
            .position(|x| x.current_payload == None && !x.taken_care_of)
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
        let mut slots = &mut self.slots;
        Dispatcher::conduct(&mut self.carriers, &mut slots);
        self.carriers.iter_mut().for_each(|x| x.tick(slots));
    }
}

#[derive(Copy, Clone)]
pub enum State {
    IDLE,
    TARGETING(usize),
    MOVING(usize),
    PICKINGUP(usize),
    LOOKINGFORTARGET,
    NOTARGET,
    DELIVERING(usize),
    PUTTINGDOWN(usize),
    _DEBUG_,
}

#[derive(Copy, Clone)]
pub struct Carrier {
    pos: Position,
    angle: f64,
    state: State,
    payload: Option<Payload>,
}

impl Carrier {
    pub fn new(x: f64, y: f64) -> Carrier {
        Carrier {
            pos: Position::new(x, y),
            angle: 0.0,
            state: State::IDLE,
            payload: None,
        }
    }

    fn target_slot(&mut self, target: usize, slot: &mut Slot) {
        self.state = State::TARGETING(target);
        slot.taken_care_of = true;
    }

    pub fn get_payload(&self) -> Option<Payload> {
        self.payload
    }

    pub fn get_target(&self) -> Option<usize> {
        match self.state {
            State::TARGETING(target_index) => Some(target_index),
            State::MOVING(target_index) => Some(target_index),
            _ => None,
        }
    }

    fn calculate_angle_to_point(&self, target: (f64, f64)) -> f64 {
        let mut angle = (target.1 - self.pos.y).atan2(target.0 - self.pos.x);
        if angle < 0.0 {
            angle += std::f64::consts::PI * 2.0;
        }
        angle
    }

    fn rotate(&mut self) {
        self.angle += ANGLE_INCREMENT;
    }

    fn rotate_to(&mut self, target_angle: f64) {
        self.rotate();
        if self.angle > std::f64::consts::PI * 2.0 {
            self.angle -= std::f64::consts::PI * 2.0;
        }
    }

    fn is_close_enough(&self, target: (f64, f64)) -> bool {
        ((self.pos.x - target.0).powf(2.0) + (self.pos.y - target.1).powf(2.0)).sqrt()
            < POSITION_EQUALITY_EPSILON
    }

    fn move_forward(&mut self) {
        self.pos.x += self.angle.cos() * SPEED_FACTOR;
        self.pos.y += self.angle.sin() * SPEED_FACTOR;
    }

    fn move_forward_to_point(&mut self, target: (f64, f64)) -> bool {
        self.move_forward();
        self.is_close_enough(target)
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

    pub fn tick(&mut self, slots: &mut Vec<Slot>) {
        match self.state {
            State::TARGETING(target) => {
                let target_pos = slots[target].get_position();
                let target_angle = self.calculate_angle_to_point((target_pos.x, target_pos.y));

                if !relative_eq!(target_angle, self.angle, epsilon = ANGLE_INCREMENT * 1.2) {
                    self.rotate_to(target_angle)
                } else {
                    self.angle = target_angle;
                    self.state = State::MOVING(target);
                }
            }
            State::MOVING(target) => {
                let target_pos = slots[target].get_position();
                if self.move_forward_to_point((target_pos.x, target_pos.y)) {
                    slots[target].taken_care_of = false;
                    match self.payload {
                        Some(_) => self.state = State::PUTTINGDOWN(target),
                        None => self.state = State::PICKINGUP(target),
                    }
                }
            }
            State::PICKINGUP(target) => {
                self.payload = slots[target].current_payload;
                slots[target].current_payload = None;
                self.state = State::LOOKINGFORTARGET;
            }
            State::PUTTINGDOWN(target) => {
                slots[target].current_payload = self.payload;
                self.payload = None;
                self.state = State::IDLE;
            }
            State::IDLE | State::NOTARGET => {
                self.move_forward();
                self.rotate();
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
    fn conduct_to_targetting() {
        let mut game = new();

        game.add_carrier(carrier!(0.0, 0.0));
        game.add_carrier(carrier!(0.0, 0.0));
        game.add_slot(slot!(100.0, 100.0, Some(Payload::from_char('X')), None));

        Dispatcher::conduct(&mut game.carriers, &mut game.slots);

        // Carrier should have target set to slot with 'X'
        let state = game.carriers[0].state;
        if let State::TARGETING(target) = state {
            assert_eq!(
                game.slots[target].current_payload,
                Some(Payload::from_char('X'))
            )
        } else {
            panic!("Found Carrier that is 'targetting' but has no target set")
        }
    }

    #[test]
    fn find_slot_for_target() {
        let mut game = new();

        game.add_slot(slot!(
            100.0,
            100.0,
            Some(Payload::from_char('X')),
            Some(Payload::from_char('Y'))
        ));
        game.add_slot(slot!(100.0, 100.0, None, Some(Payload::from_char('Z'))));
        assert_eq!(
            Dispatcher::find_slot_for_target(&game.slots, Some(Payload::from_char('Z'))),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot1() {
        let mut game = new();

        // Slot without current payload cannot have mismatched payload
        game.add_slot(slot!(100.0, 100.0, None, Some(Payload::from_char('Z'))));
        assert_eq!(
            Dispatcher::find_slot_with_mismatched_payload(&game.slots),
            None
        )
    }

    #[test]
    fn find_mismatched_slot2() {
        let mut game = new();

        game.add_slot(slot!(
            100.0,
            100.0,
            Some(Payload::from_char('A')),
            Some(Payload::from_char('Z'))
        ));
        assert_eq!(
            Dispatcher::find_slot_with_mismatched_payload(&game.slots),
            Some(0)
        )
    }

    #[test]
    fn find_mismatched_slot3() {
        let mut game = new();

        game.add_slot(slot!(
            100.0,
            100.0,
            Some(Payload::from_char('A')),
            Some(Payload::from_char('A'))
        ));
        assert_eq!(
            Dispatcher::find_slot_with_mismatched_payload(&game.slots),
            None
        )
    }
}
