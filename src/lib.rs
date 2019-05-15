mod carrier;
mod position;

use carrier::*;
use position::*;

#[macro_use]
extern crate approx;

#[macro_export]
macro_rules! make_carrier {
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
#[derive(Copy, Clone, Debug)]
pub struct Payload {
    pub cargo: char,
    taken_from: Option<usize>,
}

impl PartialEq for Payload {
    fn eq(&self, other: &Payload) -> bool {
        self.cargo == other.cargo
    }
}

impl Payload {
    pub fn from_char(c: char) -> Payload {
        Payload {
            cargo: c,
            taken_from: None,
        }
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

fn _debug_dump_slots(slots: &[Slot]) {
    for (i, v) in slots.iter().enumerate() {
        print!("Slot [{}]: ", i);

        match slots[i].current_payload {
            Some(p) => print!("{} ", p.cargo),
            None => print!("None "),
        }

        match slots[i].target_payload {
            Some(p) => print!("    {} ", p.cargo),
            None => print!("    None "),
        }

        print!("\tTaken care of={}", v.taken_care_of);

        println!();
    }
}

struct Dispatcher {}
impl Dispatcher {
    fn conduct(carriers: &mut Vec<Carrier>, slots: &mut Vec<Slot>) {
        let mut _debug_carrier_indexer = 0;
        carriers.iter_mut().for_each(|x| {
            match x.state {
                State::IDLE => {
                    if let (Some(slot_index), possible_target) =
                        Dispatcher::find_slot_with_mismatched_payload_and_free_target(slots)
                    {
                        x.target_slot(slot_index, &mut slots[slot_index]);
                        slots[possible_target].taken_care_of = true;
                        x.reserved_target = Some(possible_target);
                    } else if let Some(slot_index) =
                        Dispatcher::find_slot_with_mismatched_payload(slots)
                    {
                        x.target_slot(slot_index, &mut slots[slot_index]);
                    }
                }
                State::LOOKINGFORTARGET => match x.reserved_target {
                    Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index]),
                    None => match Dispatcher::find_slot_for_target(slots, x.payload) {
                        Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index]),
                        None => {
                            x.state = State::NOTARGET;
                        }
                    },
                },
                State::NOTARGET => match Dispatcher::find_temporary_slot(slots, x.payload) {
                    Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index]),
                    None => {
                        x.state = State::LOOKINGFORTARGET;
                    }
                },
                _ => {}
            };
            _debug_carrier_indexer += 1
        });
    }

    fn is_there_a_free_slot_for(payload: Payload, slots: &[Slot], ii: &mut usize) -> bool {
        for (i, v) in slots.iter().enumerate() {
            if v.current_payload == None
                && v.target_payload != None
                && !v.taken_care_of
                && v.target_payload.unwrap() == payload
            {
                *ii = i;
                return true;
            }
        }

        false
    }

    fn find_slot_with_mismatched_payload_and_free_target(slots: &[Slot]) -> (Option<usize>, usize) {
        let mut ii: usize = 0; // TODO: Make this an Option
        let found = slots.iter().position(|x| {
            x.current_payload != None
                && x.current_payload != x.target_payload
                && !x.taken_care_of
                && Dispatcher::is_there_a_free_slot_for(x.current_payload.unwrap(), slots, &mut ii)
        });

        (found, ii)
    }

    fn find_slot_with_mismatched_payload(slots: &[Slot]) -> Option<usize> {
        slots.iter().position(|x| {
            x.current_payload != None && x.current_payload != x.target_payload && !x.taken_care_of
        })
    }

    fn find_slot_for_target(slots: &[Slot], target: Option<Payload>) -> Option<usize> {
        let t = target.expect("Trying to find slot for empty target");

        if let Some((index, _)) = slots.iter().enumerate().find(|(index, _)| {
            slots[*index].current_payload == None
                && slots[*index].target_payload == target
                && !slots[*index].taken_care_of
                && t.taken_from != Some(*index)
        }) {
            Some(index)
        } else {
            None
        }
    }

    fn find_temporary_slot(slots: &[Slot], target: Option<Payload>) -> Option<usize> {
        let t = target.expect("Trying to find slot for empty target");

        if let Some((index, _)) = slots.iter().enumerate().find(|(index, _)| {
            slots[*index].current_payload == None
                && !slots[*index].taken_care_of
                && t.taken_from != Some(*index)
        }) {
            Some(index)
        } else {
            None
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

#[derive(Copy, Clone, Debug, PartialEq)]
enum RotationDirection {
    CLOCKWISE,
    COUNTERCLOCKWISE,
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

    #[test]
    fn rotate_direction_calculation1() {
        let mut game = new();

        game.add_carrier(carrier!(0.0, 0.0));
        let mut carrier = game.get_carriers()[0];
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI / 2.0);

        assert_eq!(
            carrier.rotation_direction.unwrap(),
            RotationDirection::CLOCKWISE
        )
    }

    #[test]
    fn rotate_direction_calculation2() {
        let mut game = new();

        game.add_carrier(carrier!(0.0, 0.0));
        let mut carrier = game.get_carriers()[0];
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI / 2.0 * 3.0);

        assert_eq!(
            carrier.rotation_direction.unwrap(),
            RotationDirection::COUNTERCLOCKWISE
        )
    }

    #[test]
    fn rotate_direction_calculation3() {
        let mut game = new();

        game.add_carrier(carrier!(0.0, 0.0));
        let mut carrier = game.get_carriers()[0];
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI);

        // When rotation 180deg, choose either left or right direction
        assert!(carrier.rotation_direction.is_some())
    }
}
