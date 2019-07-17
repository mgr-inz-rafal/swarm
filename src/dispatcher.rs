use std::collections::HashMap;
use std::hash::Hash;

use super::carrier::*;
use super::payload::*;
use super::position::*;
use super::slot::*;
use super::tools::*;

#[derive(Default)]
pub(crate) struct Dispatcher<T: PartialEq + Eq + Hash + Copy> {
    pub(crate) cargo_balance: HashMap<T, i32>,
    pub(crate) slot_distances: HashMap<(usize, usize), f64>,
}

impl<T: PartialEq + Eq + Hash + Copy> Dispatcher<T> {
    pub(crate) fn new() -> Self {
        Dispatcher {
            cargo_balance: HashMap::new(),
            slot_distances: HashMap::new(),
        }
    }

    fn calculate_cargo_balance(&mut self, slots: &[Slot<T>]) {
        self.cargo_balance.clear();

        slots.iter().for_each(|x| {
            let payloads = x.get_payloads();
            for (i, _) in payloads.iter().enumerate() {
                if let Some(payload) = payloads[i] {
                    *self.cargo_balance.entry(payload.cargo).or_insert(0) += i as i32 * -2 + 1;
                }
            }
        });
        self.cargo_balance.retain(|_, v| *v != 0);
    }

    pub(crate) fn get_distance_slot_slot(&self, s1: usize, s2: usize) -> f64 {
        *self.slot_distances.get(&(s1, s2)).unwrap()
    }

    pub(crate) fn get_distance_slot_position(
        &self,
        slots: &[Slot<T>],
        s: usize,
        pos: &Position,
    ) -> f64 {
        distance_between_positions(slots[s].get_position(), pos)
    }

    fn calculate_slot_distances(&mut self, slots: &[Slot<T>]) {
        slots.iter().enumerate().for_each(|(i1, v1)| {
            slots.iter().enumerate().for_each(|(i2, v2)| {
                self.slot_distances.insert(
                    (i1, i2),
                    distance_between_positions(v1.get_position(), v2.get_position()),
                );
            })
        });
    }

    pub(crate) fn precalc(&mut self, slots: &[Slot<T>]) {
        self.calculate_cargo_balance(slots);
        self.calculate_slot_distances(slots);
    }

    pub(crate) fn conduct(&mut self, carriers: &mut Vec<Carrier<T>>, slots: &mut Vec<Slot<T>>) {
        let mut _debug_carrier_indexer = 0;
        carriers.iter_mut().for_each(|carrier| {
            match carrier.state {
                State::MOVING(target) => {
                    if let Some(payload) = carrier.payload {
                        if carrier.temporary_target {
                            let mut ii: usize = 0;
                            let is_another_slot =
                                self.is_there_a_free_slot_for(payload, slots, &mut ii);
                            if is_another_slot && ii != target {
                                carrier.target_slot(
                                    ii,
                                    &mut slots[ii],
                                    false,
                                    false,
                                    (false, None),
                                );
                                slots[target].taken_care_of = false;
                            }
                        }
                    }
                }
                State::IDLE => {
                    if let Some(slot_index) =
                        self.find_slot_with_payload_that_should_go_to_the_pit(slots)
                    {
                        if let Some(pit_index) = self.find_closest_object(
                            slots,
                            slots[slot_index].get_position(),
                            |slot| slot.is_pit(),
                        ) {
                            carrier.target_slot(
                                slot_index,
                                &mut slots[slot_index],
                                false,
                                true,
                                (false, None),
                            );
                            carrier.reserved_target = Some(pit_index);
                            self.reduce_cargo_balance(
                                slots[slot_index].get_payloads()[0].unwrap().cargo,
                            );
                        }
                    } else if let (Some(slot_index), possible_target) =
                        self.find_slot_with_mismatched_payload_and_free_target(slots)
                    {
                        carrier.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            false,
                            false,
                            (false, None),
                        );
                        slots[possible_target].taken_care_of = true;
                        carrier.reserved_target = Some(possible_target);
                    } else if let Some(slot_index) = self.find_slot_with_mismatched_payload(slots) {
                        carrier.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            false,
                            false,
                            (false, None),
                        );
                    } else if let Some(cargo) = self.get_cargo_to_spawn() {
                        if let Some(slot_index) =
                            self.find_closest_object(slots, carrier.get_position(), |slot| {
                                slot.is_spawner()
                            })
                        {
                            carrier.target_slot(
                                slot_index,
                                &mut slots[slot_index],
                                false,
                                false,
                                (true, Some(cargo)),
                            );
                        }
                    }
                }
                State::LOOKINGFORTARGET => match carrier.reserved_target {
                    Some(slot_index) => carrier.target_slot(
                        slot_index,
                        &mut slots[slot_index],
                        carrier.temporary_target,
                        carrier.carrying_to_pit,
                        carrier.going_to_spawner,
                    ),
                    None => match self.find_slot_for_target(slots, carrier.payload) {
                        Some(slot_index) => carrier.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            carrier.temporary_target,
                            carrier.carrying_to_pit,
                            carrier.going_to_spawner,
                        ),
                        None => {
                            carrier.state = State::NOTARGET;
                        }
                    },
                },
                State::NOTARGET => match self.find_temporary_slot(slots, carrier.payload) {
                    Some(slot_index) => {
                        carrier.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            true,
                            false,
                            (false, None),
                        );
                    }
                    None => {
                        carrier.state = State::LOOKINGFORTARGET;
                    }
                },
                _ => {}
            };
            _debug_carrier_indexer += 1
        });
    }

    fn get_cargo_to_spawn(&mut self) -> Option<T> {
        if self.cargo_balance.is_empty() {
            return None;
        }
        let missing = *self.cargo_balance.keys().next().unwrap();
        self.increase_cargo_balance(missing);
        Some(missing)
    }

    fn reduce_cargo_balance(&mut self, cargo: T) {
        *self.cargo_balance.entry(cargo).or_insert(0) -= 1;
        self.cargo_balance.retain(|_, v| *v != 0);
    }

    fn increase_cargo_balance(&mut self, cargo: T) {
        *self.cargo_balance.entry(cargo).or_insert(0) += 1;
        self.cargo_balance.retain(|_, v| *v != 0);
    }

    fn find_closest_object(
        &self,
        slots: &[Slot<T>],
        pos: &Position,
        classifier: fn(&Slot<T>) -> bool,
    ) -> Option<usize> {
        let mut distances = Vec::new();
        slots.iter().enumerate().for_each(|(i, v)| {
            if classifier(&v) {
                distances.push((i, self.get_distance_slot_position(slots, i, pos)));
            }
        });
        if distances.is_empty() {
            return None;
        };
        Some(
            distances
                .iter()
                .min_by(|a, b| (a.1).partial_cmp(&b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn find_slot_with_payload_that_should_go_to_the_pit(&self, slots: &[Slot<T>]) -> Option<usize> {
        let excessive = self.cargo_balance.iter().find(|&(_, &v)| v > 0);
        if let Some(cargo) = excessive {
            if let Some(slot_index) = self.find_mismatched_slot_that_contains(slots, *cargo.0) {
                if !slots[slot_index].taken_care_of {
                    return Some(slot_index);
                }
            }
        }
        None
    }

    fn find_mismatched_slot_that_contains(&self, slots: &[Slot<T>], cargo: T) -> Option<usize> {
        for (i, v) in slots.iter().enumerate() {
            if !v.taken_care_of {
                let [current, target] = v.get_payloads();
                if current != target {
                    if let Some(contained_cargo) = current {
                        if contained_cargo.cargo == cargo {
                            return Some(i);
                        }
                    }
                }
            }
        }
        None
    }

    fn is_there_a_free_slot_for(
        &self,
        payload: Payload<T>,
        slots: &[Slot<T>],
        ii: &mut usize,
    ) -> bool {
        for (i, v) in slots.iter().enumerate() {
            let [current, target] = v.get_payloads();
            if current == None && target != None && !v.taken_care_of && target.unwrap() == payload {
                *ii = i;
                return true;
            }
        }

        false
    }

    fn find_slot_with_mismatched_payload_and_free_target(
        &self,
        slots: &[Slot<T>],
    ) -> (Option<usize>, usize) {
        let mut ii: usize = 0; // TODO: Make this an Option
        let found = slots.iter().position(|x| {
            let [current, target] = x.get_payloads();
            current != None
                && current != target
                && !x.taken_care_of
                && self.is_there_a_free_slot_for(current.unwrap(), slots, &mut ii)
        });

        (found, ii)
    }

    fn find_slot_with_mismatched_payload(&self, slots: &[Slot<T>]) -> Option<usize> {
        slots.iter().position(|x| {
            let [current, target] = x.get_payloads();
            current != None && current != target && !x.taken_care_of
        })
    }

    fn find_slot_for_target(
        &self,
        slots: &[Slot<T>],
        target_payload: Option<Payload<T>>,
    ) -> Option<usize> {
        let t = target_payload.expect("Trying to find slot for empty target");

        if let Some((index, _)) = slots.iter().enumerate().find(|(index, _)| {
            let [current, _] = slots[*index].get_payloads();
            current == None
                && slots[*index].accepts(target_payload)
                && !slots[*index].taken_care_of
                && t.taken_from != Some(*index)
        }) {
            Some(index)
        } else {
            None
        }
    }

    fn is_candidate_for_temporary_slot(
        &self,
        slots: &[Slot<T>],
        index: usize,
        target: Payload<T>,
    ) -> bool {
        slots[index].current_payload == None
            && !slots[index].is_pit()
            && !slots[index].is_spawner()
            && !slots[index].taken_care_of
            && target.taken_from != Some(index)
    }

    fn _find_any_temporary_slot(&self, slots: &[Slot<T>], target: Payload<T>) -> Option<usize> {
        if let Some((index, _)) = slots
            .iter()
            .enumerate()
            .find(|(index, _)| self.is_candidate_for_temporary_slot(slots, *index, target))
        {
            Some(index)
        } else {
            None
        }
    }

    fn find_closest_temporary_slot(&self, slots: &[Slot<T>], target: Payload<T>) -> Option<usize> {
        let mut distances = Vec::new();
        slots.iter().enumerate().for_each(|(i, _)| {
            if self.is_candidate_for_temporary_slot(slots, i, target) {
                distances.push((
                    i,
                    self.get_distance_slot_slot(i, target.taken_from.unwrap()),
                ));
            }
        });
        if distances.is_empty() {
            return None;
        };
        Some(
            distances
                .iter()
                .min_by(|a, b| (a.1).partial_cmp(&b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn find_temporary_slot(&self, slots: &[Slot<T>], target: Option<Payload<T>>) -> Option<usize> {
        let t = target.expect("Trying to find slot for empty target");
        self.find_closest_temporary_slot(slots, t)
    }
}

#[cfg(test)]
mod tests {
    use crate::dispatcher::*;

    #[test]
    fn find_slot_for_target() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_for_target(&slots, Some(Payload::new('B'))),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot1() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('X')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('Y')),
                SlotKind::CLASSIC,
            ),
        ];

        // Slot without current payload cannot have mismatched payload
        assert_eq!(dispatcher.find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot2() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('X')),
                Some(Payload::new('X')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('Y')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload(&slots),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot3() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('Y')),
                Some(Payload::new('Y')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(dispatcher.find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot_with_target1() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('X')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target2() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target3() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (Some(0), 1)
        )
    }

    #[test]
    fn is_there_a_free_slot_for1() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
        ];

        let p = Payload::new('C');
        let mut ii = 0;

        assert_eq!(
            dispatcher.is_there_a_free_slot_for(p, &slots, &mut ii),
            true
        );
        assert_eq!(ii, 1)
    }

    #[test]
    fn is_there_a_free_slot_for2() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('D')),
                SlotKind::CLASSIC,
            ),
        ];

        let p = Payload::new('C');
        let mut ii = 0;

        assert_eq!(
            dispatcher.is_there_a_free_slot_for(p, &slots, &mut ii),
            false
        );
    }

    #[test]
    fn calculate_cargo_balance() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('C')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('E')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('F')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
        ];

        dispatcher.calculate_cargo_balance(&slots);

        assert_eq!(dispatcher.cargo_balance.get(&'A'), None);
        assert_eq!(dispatcher.cargo_balance.get(&'C'), None);
        assert_eq!(dispatcher.cargo_balance[&'E'], 1);
        assert_eq!(dispatcher.cargo_balance[&'F'], -1);
        assert_eq!(dispatcher.cargo_balance[&'G'], -2);
    }

    #[test]
    fn reduce_cargo_balance() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('C')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('E')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('F')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
        ];

        dispatcher.calculate_cargo_balance(&slots);
        dispatcher.reduce_cargo_balance('G');
        dispatcher.reduce_cargo_balance('E');

        assert_eq!(dispatcher.cargo_balance.get(&'A'), None);
        assert_eq!(dispatcher.cargo_balance.get(&'C'), None);
        assert_eq!(dispatcher.cargo_balance.get(&'E'), None);
        assert_eq!(dispatcher.cargo_balance[&'F'], -1);
        assert_eq!(dispatcher.cargo_balance[&'G'], -3);
    }

    #[test]
    fn find_closest_object1() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_pit()),
            None
        );
        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_spawner()),
            None
        );
    }

    #[test]
    fn find_closest_object() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            make_slot_pit!(1000.0, 1000.0),
            make_slot_spawner!(1000.0, 1000.0),
        ];

        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_pit()),
            Some(1)
        );
        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_spawner()),
            Some(2)
        );
    }

    #[test]
    fn find_closest_object3() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            make_slot_pit!(1000.0, 1000.0),
            make_slot_pit!(0.0, 0.0),
            make_slot_spawner!(1000.0, 1000.0),
            make_slot_spawner!(0.0, 0.0),
        ];

        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_pit()),
            Some(2)
        );
        assert_eq!(
            dispatcher
                .find_closest_object(&slots, &Position::new(10.0, 10.0), |slot| slot.is_spawner()),
            Some(4)
        );
    }

    #[test]
    fn find_slot_with_payload_that_should_go_to_the_pit1() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('C')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('E')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('F')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
        ];

        dispatcher.calculate_cargo_balance(&slots);
        assert_ne!(
            dispatcher.find_slot_with_payload_that_should_go_to_the_pit(&slots),
            Some(2)
        );
        assert_eq!(
            dispatcher.find_slot_with_payload_that_should_go_to_the_pit(&slots),
            Some(3)
        );
    }

    #[test]
    fn find_slot_with_payload_that_should_go_to_the_pit2() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('C')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('C')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('E')),
                Some(Payload::new('E')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('G')),
                SlotKind::CLASSIC,
            ),
        ];

        dispatcher.calculate_cargo_balance(&slots);
        assert_eq!(
            dispatcher.find_slot_with_payload_that_should_go_to_the_pit(&slots),
            None
        );
    }

    #[test]
    fn find_slot_that_contains() {
        let dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('X')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                None,
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
        ];

        assert_eq!(
            dispatcher.find_mismatched_slot_that_contains(&slots, 'A'),
            Some(2)
        );
        assert_eq!(
            dispatcher.find_mismatched_slot_that_contains(&slots, 'X'),
            Some(0)
        );
        assert_eq!(
            dispatcher.find_mismatched_slot_that_contains(&slots, 'Y'),
            None
        );
    }

    #[test]
    fn calculate_slot_distances() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                100.0,
                100.0,
                Some(Payload::new('A')),
                None,
                SlotKind::CLASSIC,
            ),
            Slot::new(100.0, 100.0, None, None, SlotKind::CLASSIC),
            Slot::new(200.0, 200.0, None, None, SlotKind::CLASSIC),
        ];

        dispatcher.calculate_slot_distances(&slots);

        assert!(relative_eq!(dispatcher.get_distance_slot_slot(0, 1), 0.0));
        assert!(relative_eq!(
            dispatcher.get_distance_slot_slot(1, 2),
            100.0 * (2.0 as f64).sqrt()
        ));
        assert!(relative_eq!(
            dispatcher.get_distance_slot_slot(2, 1),
            dispatcher.get_distance_slot_slot(1, 2)
        ))
    }

    #[test]
    fn find_any_temporary_slot() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                200.0,
                210.0,
                Some(Payload::new('B')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                200.0,
                300.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(600.0, 550.0, None, None, SlotKind::CLASSIC),
            Slot::new(500.0, 450.0, None, None, SlotKind::CLASSIC),
            Slot::new(300.0, 350.0, None, None, SlotKind::CLASSIC),
        ];

        dispatcher.calculate_slot_distances(&slots);
        let tmp_slot = dispatcher
            ._find_any_temporary_slot(&slots, Payload::new('A'))
            .unwrap();
        assert_eq!(tmp_slot, 2);
    }

    #[test]
    fn find_closest_temporary_slot() {
        let mut dispatcher = Dispatcher::new();
        let slots = vec![
            Slot::new(
                200.0,
                210.0,
                Some(Payload::new('B')),
                Some(Payload::new('A')),
                SlotKind::CLASSIC,
            ),
            Slot::new(
                200.0,
                300.0,
                Some(Payload::new('A')),
                Some(Payload::new('B')),
                SlotKind::CLASSIC,
            ),
            Slot::new(600.0, 550.0, None, None, SlotKind::CLASSIC),
            Slot::new(500.0, 450.0, None, None, SlotKind::CLASSIC),
            Slot::new(300.0, 350.0, None, None, SlotKind::CLASSIC),
        ];

        let mut payload = Payload::new('A');
        payload.taken_from = Some(1);

        dispatcher.calculate_slot_distances(&slots);
        let tmp_slot = dispatcher
            .find_closest_temporary_slot(&slots, payload)
            .unwrap();
        assert_eq!(tmp_slot, 4);
    }
}
