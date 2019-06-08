#![macro_use]

extern crate rand;

use super::payload::*;
use super::position::*;
use super::slot::*;
use rand::Rng;
use std::hash::Hash;

const ANGLE_INCREMENT: f64 = 0.15;
const SPEED_FACTOR: f64 = 6.0;
const POSITION_EQUALITY_EPSILON: f64 = SPEED_FACTOR * 1.5;

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
pub(crate) enum RotationDirection {
    CLOCKWISE,
    COUNTERCLOCKWISE,
}

#[derive(Copy, Clone)]
pub struct Carrier<T: PartialEq + Eq + Hash + Copy> {
    pos: Position,
    angle: f64,
    pub(crate) state: State,
    pub(crate) payload: Option<Payload<T>>,
    pub(crate) reserved_target: Option<usize>,
    rotation_direction: Option<RotationDirection>,
    idle_rotation_direction: Option<RotationDirection>,
    pub(crate) temporary_target: bool,
    pub(crate) carrying_to_pit: bool,
    pub(crate) going_to_spawner: (bool, Option<T>),
}

impl<T: PartialEq + Eq + Hash + Copy> Carrier<T> {
    pub fn new(x: f64, y: f64) -> Carrier<T> {
        Carrier {
            pos: Position::new(x, y),
            angle: 0.0,
            state: State::IDLE,
            payload: None,
            reserved_target: None,
            rotation_direction: None,
            idle_rotation_direction: Carrier::<T>::pick_random_idle_rotation(),
            temporary_target: false,
            carrying_to_pit: false,
            going_to_spawner: (false, None),
        }
    }

    fn pick_random_idle_rotation() -> Option<RotationDirection> {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0, 2) {
            0 => Some(RotationDirection::CLOCKWISE),
            1 => Some(RotationDirection::COUNTERCLOCKWISE),
            _ => panic!("Random generator bug?"),
        }
    }

    pub(crate) fn target_slot(
        &mut self,
        target: usize,
        slot: &mut Slot<T>,
        is_temporary: bool,
        to_pit: bool,
        to_spawner: (bool, Option<T>),
    ) {
        if slot.is_pit() && self.get_payload().is_none() {
            panic!("Going empty to the pit, will try to pickup");
        }

        self.state = State::TARGETING(target);
        slot.taken_care_of = true;
        self.rotation_direction = None;
        self.temporary_target = is_temporary;
        self.carrying_to_pit = to_pit;
        self.going_to_spawner = to_spawner;
    }

    pub fn get_payload(&self) -> Option<Payload<T>> {
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
        if let Some(direction) = self.rotation_direction {
            match direction {
                RotationDirection::CLOCKWISE => self.angle += ANGLE_INCREMENT,
                RotationDirection::COUNTERCLOCKWISE => self.angle -= ANGLE_INCREMENT,
            }
        }
    }

    fn idle_rotate(&mut self) {
        if let Some(direction) = self.idle_rotation_direction {
            match direction {
                RotationDirection::CLOCKWISE => self.angle += ANGLE_INCREMENT,
                RotationDirection::COUNTERCLOCKWISE => self.angle -= ANGLE_INCREMENT,
            }
        }
    }

    fn rotate_to(&mut self, target_angle: f64) {
        if self.rotation_direction.is_none() {
            let src = self.angle;
            let mut trg = target_angle;
            if trg < src {
                trg += std::f64::consts::PI * 2.0
            };
            self.rotation_direction = if trg - src > std::f64::consts::PI {
                Some(RotationDirection::COUNTERCLOCKWISE)
            } else {
                Some(RotationDirection::CLOCKWISE)
            };
        }

        self.rotate();
        if let Some(direction) = self.rotation_direction {
            match direction {
                RotationDirection::CLOCKWISE => {
                    if self.angle > std::f64::consts::PI * 2.0 {
                        self.angle -= std::f64::consts::PI * 2.0;
                    }
                }
                RotationDirection::COUNTERCLOCKWISE => {
                    if self.angle < 0.0 {
                        self.angle += std::f64::consts::PI * 2.0;
                    }
                }
            }
        }
    }

    fn is_close_enough(&self, target: (f64, f64)) -> bool {
        relative_eq!(
            ((self.pos.x - target.0).powf(2.0) + (self.pos.y - target.1).powf(2.0)).sqrt(),
            0.0,
            epsilon = POSITION_EQUALITY_EPSILON
        )
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

    pub fn tick(&mut self, slots: &mut Vec<Slot<T>>) {
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
                    self.rotation_direction = None;
                    match self.payload {
                        Some(_) => self.state = State::PUTTINGDOWN(target),
                        None => self.state = State::PICKINGUP(target),
                    }
                }
            }
            State::PICKINGUP(target) => {
                if slots[target].is_pit() {
                    panic!("Trying to pick up from the pit");
                }

                self.payload = if self.going_to_spawner.0 {
                    Some(Payload {
                        cargo: self.going_to_spawner.1.unwrap(),
                        taken_from: None,
                    })
                } else {
                    slots[target].current_payload
                };

                if self.payload.is_some() {
                    self.payload = Some(Payload {
                        taken_from: Some(target),
                        cargo: if self.going_to_spawner.0 {
                            self.going_to_spawner.1.unwrap()
                        } else {
                            slots[target].current_payload.unwrap().cargo
                        },
                    });
                    slots[target].current_payload = None;
                    slots[target].taken_care_of = false;
                    self.state = State::LOOKINGFORTARGET;
                } else {
                    panic!("Want to pick up from slot without payload")
                }
            }
            State::PUTTINGDOWN(target) => {
                if slots[target].is_spawner() {
                    panic!("Trying to drop into the spawner");
                }
                if !self.carrying_to_pit {
                    slots[target].current_payload = self.payload;
                    slots[target].taken_care_of = false;
                }
                self.reserved_target = None;
                self.payload = None;
                self.state = State::IDLE;
                self.idle_rotation_direction = Carrier::<T>::pick_random_idle_rotation();
            }
            State::IDLE | State::NOTARGET => {
                self.move_forward();
                self.idle_rotate();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::carrier::*;
    #[test]
    fn rotate_direction_calculation1() {
        let mut carrier = Carrier::<usize>::new(0.0, 0.0);
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI / 2.0);

        assert_eq!(
            carrier.rotation_direction.unwrap(),
            RotationDirection::CLOCKWISE
        )
    }

    #[test]
    fn rotate_direction_calculation2() {
        let mut carrier = Carrier::<usize>::new(0.0, 0.0);
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI / 2.0 * 3.0);

        assert_eq!(
            carrier.rotation_direction.unwrap(),
            RotationDirection::COUNTERCLOCKWISE
        )
    }

    #[test]
    fn rotate_direction_calculation3() {
        let mut carrier = Carrier::<usize>::new(0.0, 0.0);
        carrier.angle = 0.0;
        carrier.rotate_to(std::f64::consts::PI);

        // When rotating 180deg, choose either left or right direction
        assert!(carrier.rotation_direction.is_some())
    }

}
