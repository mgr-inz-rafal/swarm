#![macro_use]
extern crate rand;

use super::payload::*;
use super::position::*;
use super::slot::*;
use super::tools::*;
use rand::Rng;
use std::hash::Hash;

const ANGLE_INCREMENT: f64 = 0.15;
const DEFAULT_ACCELERATION: f64 = 0.47;
const DEFAULT_MAX_SPEED: f64 = 6.0;

/// States that apply to Carriers
///
/// State            | Meaning
/// -----------------|--------
/// IDLE             | Not doing anything, except for looking for a new task
/// TARGETING        | Rotating to face the current target
/// MOVING           | Moving to target
/// PICKINGUP        | Picking up the paylaod
/// LOOKINGFORTARGET | Looking for target for the payload
/// NOTARGET         | Has payload that currently won't fit anywhere. Will be temporarily dropped in the closest slot
/// DELIVERING       | Moving payload to the target
/// PUTTINGDOWN      | Putting down the payload
#[derive(Copy, Clone, PartialEq, Debug)]
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

impl State {
    pub(crate) fn is_idle(&self) -> bool {
        match *self {
            State::IDLE => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum RotationDirection {
    CLOCKWISE,
    COUNTERCLOCKWISE,
}

/// Represnets the `Carrier` object. Carrier is an entity that moves from slot to slot and
/// transfers payload in order to reach the desired layout.
#[derive(Copy, Clone)]
pub struct Carrier<T: PartialEq + Eq + Hash + Copy> {
    pos: Position,
    angle: f64,
    acceleration: f64,
    effective_acceleration: f64,
    max_speed: f64,
    speed: f64,
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
    /// Creates new Carrier at the position specified
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// ```
    pub fn new(x: f64, y: f64) -> Carrier<T> {
        Carrier {
            pos: Position::new(x, y),
            angle: 0.0,
            acceleration: DEFAULT_ACCELERATION,
            effective_acceleration: 0.0,
            max_speed: DEFAULT_MAX_SPEED,
            speed: 0.0,
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

    /// Returns current payload of the carrier
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// let payload = carrier.get_payload();
    /// assert_eq!(payload, None);
    ///
    /// ```
    pub fn get_payload(&self) -> Option<Payload<T>> {
        self.payload
    }

    /// Returns index of the slot that carriers is going to
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// let target = carrier.get_target();
    /// assert_eq!(target, None)
    /// ```
    pub fn get_target(&self) -> Option<usize> {
        match self.state {
            State::TARGETING(target_index) => Some(target_index),
            State::MOVING(target_index) => Some(target_index),
            _ => None,
        }
    }

    /// Returns index of the slot that is reserverd as a
    /// target for the cargo that the carrier is going to pick-up
    /// or is currently transferring.
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// let target = carrier.get_reserved_target();
    /// assert_eq!(target, None)
    /// ```
    pub fn get_reserved_target(&self) -> Option<usize> {
        self.reserved_target
    }

    /// Returns current carrier position
    ///
    /// # Example
    ///
    /// ```
    /// let x = 100.0;
    /// let y = 200.0;
    /// let carrier = swarm_it::Carrier::<char>::new(x, y);
    /// let position = carrier.get_position();
    /// approx::assert_abs_diff_eq!(position.x, x);
    /// approx::assert_abs_diff_eq!(position.y, y);
    /// ```
    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Returns current carrier angle.
    ///
    /// Direction | Angle
    /// ----------|--------
    /// Right     | 0 / 2*π
    /// Down      | π/2
    /// Left      | π
    /// Up        | ³/₂*π
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// approx::assert_abs_diff_eq!(carrier.get_angle(), 0.0);
    /// ```
    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    /// Returns current carrier state
    ///
    /// # Example
    ///
    /// ```
    /// let carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// assert_eq!(carrier.get_state(), swarm_it::State::IDLE);
    /// ```
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Sets carrier acceleration. This *doesn't* affect the current movement
    /// of a carrier, since this is controlled by the engine. It sets the
    /// carrier's ability to accelerate.
    ///
    /// # Example
    ///
    /// ```
    /// let mut carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// carrier.set_acceleration(2.71);
    /// approx::assert_abs_diff_eq!(carrier.get_acceleration(), 2.71);
    /// ```
    pub fn set_acceleration(&mut self, acceleration: f64) {
        self.acceleration = acceleration;
    }

    /// Gets carrier acceleration. This *doesn't* return the current
    /// acceleration, but the number that defines the carrier's ability
    /// to accelerate.
    ///
    /// # Example
    ///
    /// ```
    /// let mut carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// carrier.set_acceleration(2.71);
    /// approx::assert_abs_diff_eq!(carrier.get_acceleration(), 2.71);
    /// ```
    pub fn get_acceleration(&self) -> f64 {
        self.acceleration
    }

    /// Sets maximum speed of a carrier.
    ///
    /// # Example
    ///
    /// ```
    /// let mut carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// carrier.set_max_speed(3.01);
    /// approx::assert_abs_diff_eq!(carrier.get_max_speed(), 3.01);
    /// ```
    pub fn set_max_speed(&mut self, max_speed: f64) {
        self.max_speed = max_speed;
    }

    /// Returns maximum speed of a carrier.
    ///
    /// # Example
    ///
    /// ```
    /// let mut carrier = swarm_it::Carrier::<char>::new(100.0, 100.0);
    /// carrier.set_max_speed(3.01);
    /// approx::assert_abs_diff_eq!(carrier.get_max_speed(), 3.01);
    /// ```
    pub fn get_max_speed(&self) -> f64 {
        self.max_speed
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
        self.speed = 0.0;
        self.effective_acceleration = self.acceleration;
        slot.taken_care_of = true;
        self.rotation_direction = None;
        self.temporary_target = is_temporary;
        self.carrying_to_pit = to_pit;
        self.going_to_spawner = to_spawner;
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
                RotationDirection::CLOCKWISE => self.turn_clockwise(),
                RotationDirection::COUNTERCLOCKWISE => self.turn_counterclockwise(),
            }
        }
    }

    fn idle_rotate(&mut self) {
        if let Some(direction) = self.idle_rotation_direction {
            match direction {
                RotationDirection::CLOCKWISE => self.turn_clockwise(),
                RotationDirection::COUNTERCLOCKWISE => self.turn_counterclockwise(),
            }
        }
    }

    fn turn_clockwise(&mut self) {
        self.angle += ANGLE_INCREMENT;
        if self.angle > 2.0 * std::f64::consts::PI {
            self.angle -= 2.0 * std::f64::consts::PI;
        }
    }

    fn turn_counterclockwise(&mut self) {
        self.angle -= ANGLE_INCREMENT;
        if self.angle < 0.0 {
            self.angle += 2.0 * std::f64::consts::PI;
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

    fn accelerate(&mut self) -> bool {
        self.speed += self.effective_acceleration;
        if self.speed > self.max_speed {
            self.speed = self.max_speed
        };
        if self.speed < 0.0 {
            self.speed = 0.0;
            return true; // Notify that carrier is at the destination
        };
        false
    }

    fn calculate_tics_to_decelerate(&self) -> u32 {
        let mut ticks_to_decelerate = 0;
        let mut speed_tmp = self.speed;
        loop {
            speed_tmp -= self.acceleration;
            ticks_to_decelerate += 1;
            if speed_tmp < 0.0 {
                return ticks_to_decelerate;
            }
        }
    }

    fn calculate_distance_to_stop(&self) -> f64 {
        let mut distance_to_stop = 0.0;
        let mut speed_tmp = self.speed;
        for _ in 0..self.calculate_tics_to_decelerate() {
            distance_to_stop += speed_tmp;
            speed_tmp -= self.acceleration;
        }
        distance_to_stop
    }

    fn move_forward(&mut self, target: (f64, f64)) -> bool {
        if self.effective_acceleration > 0.0 {
            let distance_to_stop = self.calculate_distance_to_stop();
            let distance_to_target =
                distance_between_positions(&Position::new(target.0, target.1), self.get_position());
            if distance_to_stop > distance_to_target {
                self.effective_acceleration = -self.effective_acceleration;
            }
        }

        let at_destination = self.accelerate();
        self.pos.x += self.angle.cos() * self.speed;
        self.pos.y += self.angle.sin() * self.speed;
        at_destination
    }

    fn move_forward_to_point(&mut self, target: (f64, f64)) -> bool {
        self.move_forward(target)
    }

    pub(crate) fn tick(&mut self, slots: &mut Vec<Slot<T>>) {
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
                    self.effective_acceleration = self.acceleration;
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
                self.move_forward((5.0, 5.0)); // TODO: Allow moving forward without specifying target
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
