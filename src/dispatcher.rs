use std::collections::HashMap;
use std::hash::Hash;

use super::carrier::*;
use super::payload::*;
use super::slot::*;

// TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
pub struct Dispatcher<T: PartialEq + Eq + Hash + Copy> {
    pub(crate) cargo_balance: HashMap<T, i32>,
}

impl<T: PartialEq + Eq + Hash + Copy> Dispatcher<T> {
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

    pub(crate) fn precalc(&mut self, slots: &[Slot<T>]) {
        self.calculate_cargo_balance(slots);
    }

    pub(crate) fn conduct(&mut self, carriers: &mut Vec<Carrier<T>>, slots: &mut Vec<Slot<T>>) {
        let mut _debug_carrier_indexer = 0;
        carriers.iter_mut().for_each(|x| {
            match x.state {
                State::MOVING(target) => {
                    if let Some(payload) = x.payload {
                        if x.temporary_target {
                            let mut ii: usize = 0;
                            let is_another_slot =
                                self.is_there_a_free_slot_for(payload, slots, &mut ii);
                            if is_another_slot && ii != target {
                                x.target_slot(ii, &mut slots[ii], false, false, (false, None));
                                slots[target].taken_care_of = false;
                            }
                        }
                    }
                }
                State::IDLE => {
                    if let Some(slot_index) =
                        self.find_slot_with_payload_that_should_go_to_the_pit(slots)
                    {
                        if let Some(pit_index) = self.find_pit(slots) {
                            x.target_slot(
                                slot_index,
                                &mut slots[slot_index],
                                false,
                                true,
                                (false, None),
                            );
                            x.reserved_target = Some(pit_index);
                            self.reduce_cargo_balance(
                                slots[slot_index].get_payloads()[0].unwrap().cargo,
                            );
                        }
                    } else if let (Some(slot_index), possible_target) =
                        self.find_slot_with_mismatched_payload_and_free_target(slots)
                    {
                        x.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            false,
                            false,
                            (false, None),
                        );
                        slots[possible_target].taken_care_of = true;
                        x.reserved_target = Some(possible_target);
                    } else if let Some(slot_index) = self.find_slot_with_mismatched_payload(slots) {
                        x.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            false,
                            false,
                            (false, None),
                        );
                    } else if let Some(cargo) = self.get_cargo_to_spawn() {
                        if let Some(slot_index) = self.find_spawner(&slots) {
                            x.target_slot(
                                slot_index,
                                &mut slots[slot_index],
                                false,
                                false,
                                (true, Some(cargo)),
                            );
                        }
                    }
                }
                State::LOOKINGFORTARGET => match x.reserved_target {
                    Some(slot_index) => x.target_slot(
                        slot_index,
                        &mut slots[slot_index],
                        x.temporary_target,
                        x.carrying_to_pit,
                        x.going_to_spawner,
                    ),
                    None => match self.find_slot_for_target(slots, x.payload) {
                        Some(slot_index) => x.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            x.temporary_target,
                            x.carrying_to_pit,
                            x.going_to_spawner,
                        ),
                        None => {
                            x.state = State::NOTARGET;
                        }
                    },
                },
                State::NOTARGET => match self.find_temporary_slot(slots, x.payload) {
                    Some(slot_index) => {
                        x.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            true,
                            false,
                            (false, None),
                        );
                    }
                    None => {
                        x.state = State::LOOKINGFORTARGET;
                    }
                },
                _ => {}
            };
            _debug_carrier_indexer += 1
        });
    }

    // TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
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

    fn find_pit(&self, slots: &[Slot<T>]) -> Option<usize> {
        for (i, v) in slots.iter().enumerate() {
            if v.is_pit() {
                return Some(i);
            }
        }
        None
    }

    // TODO: Merge with find_pit
    fn find_spawner(&self, slots: &[Slot<T>]) -> Option<usize> {
        for (i, v) in slots.iter().enumerate() {
            if v.is_spawner() {
                return Some(i);
            }
        }
        None
    }

    fn find_slot_with_payload_that_should_go_to_the_pit(&self, slots: &[Slot<T>]) -> Option<usize> {
        let excessive = self.cargo_balance.iter().find(|&(_, &v)| v > 0);
        if let Some(cargo) = excessive {
            if let Some(slot_index) = self.find_slot_that_contains(slots, *cargo.0) {
                if !slots[slot_index].taken_care_of {
                    return Some(slot_index);
                }
            }
        }
        None
    }

    // TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
    fn find_slot_that_contains(&self, slots: &[Slot<T>], cargo: T) -> Option<usize> {
        for (i, v) in slots.iter().enumerate() {
            let [current, _] = v.get_payloads();
            if let Some(contained_cargo) = current {
                if contained_cargo.cargo == cargo {
                    return Some(i);
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

    fn find_temporary_slot(&self, slots: &[Slot<T>], target: Option<Payload<T>>) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use crate::dispatcher::*;

    #[test]
    fn find_slot_for_target() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('X')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('B'))),
        ];

        assert_eq!(
            dispatcher.find_slot_for_target(&slots, Some(Payload::from_char('B'))),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot1() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('X'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('Y'))),
        ];

        // Slot without current payload cannot have mismatched payload
        assert_eq!(dispatcher.find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot2() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('X')),
                Some(Payload::from_char('X'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('Y'))
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload(&slots),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot3() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('Y')),
                Some(Payload::from_char('Y'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('A'))
            ),
        ];

        assert_eq!(dispatcher.find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot_with_target1() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('X')),
                Some(Payload::from_char('B'))
            ),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target2() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('B'))),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target3() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('A'))),
        ];

        assert_eq!(
            dispatcher.find_slot_with_mismatched_payload_and_free_target(&slots),
            (Some(0), 1)
        )
    }

    #[test]
    fn is_there_a_free_slot_for1() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('C'))),
        ];

        let p = Payload::from_char('C');
        let mut ii = 0;

        assert_eq!(
            dispatcher.is_there_a_free_slot_for(p, &slots, &mut ii),
            true
        );
        assert_eq!(ii, 1)
    }

    #[test]
    fn is_there_a_free_slot_for2() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('C'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('D'))),
        ];

        let p = Payload::from_char('C');
        let mut ii = 0;

        assert_eq!(
            dispatcher.is_there_a_free_slot_for(p, &slots, &mut ii),
            false
        );
    }

    #[test]
    fn calculate_cargo_balance() {
        let mut dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('C'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('C')),
                Some(Payload::from_char('A'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('E'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('F'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
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
        let mut dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('C'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('C')),
                Some(Payload::from_char('A'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('E'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('F'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
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
    fn find_pit1() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('C'))),
        ];

        assert_eq!(dispatcher.find_pit(&slots), None);
    }

    #[test]
    fn find_pit2() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
            make_slot_pit!(100.0, 100.0),
        ];

        assert_eq!(dispatcher.find_pit(&slots), Some(1));
    }

    #[test]
    fn find_slot_with_payload_that_should_go_to_the_pit1() {
        let mut dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('C'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('C')),
                Some(Payload::from_char('A'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('E'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('F'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
        ];

        dispatcher.calculate_cargo_balance(&slots);
        assert_eq!(
            dispatcher.find_slot_with_payload_that_should_go_to_the_pit(&slots),
            Some(2)
        );
    }

    #[test]
    fn find_slot_with_payload_that_should_go_to_the_pit2() {
        let mut dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('C'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('C')),
                Some(Payload::from_char('A'))
            ),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('E')),
                Some(Payload::from_char('E'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('G'))),
        ];

        dispatcher.calculate_cargo_balance(&slots);
        assert_eq!(
            dispatcher.find_slot_with_payload_that_should_go_to_the_pit(&slots),
            None
        );
    }

    #[test]
    fn find_slot_that_contains() {
        let dispatcher = Dispatcher {
            cargo_balance: HashMap::new(),
        };
        let slots = vec![
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('X')),
                Some(Payload::from_char('B'))
            ),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('B'))),
            make_slot!(
                100.0,
                100.0,
                Some(Payload::from_char('A')),
                Some(Payload::from_char('B'))
            ),
        ];

        assert_eq!(dispatcher.find_slot_that_contains(&slots, 'A'), Some(2));
        assert_eq!(dispatcher.find_slot_that_contains(&slots, 'X'), Some(0));
        assert_eq!(dispatcher.find_slot_that_contains(&slots, 'Y'), None);
    }

}
