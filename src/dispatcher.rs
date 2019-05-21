use std::collections::HashMap;

use super::carrier::*;
use super::payload::*;
use super::slot::*;

// TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
pub struct Dispatcher {
    pub(crate) cargo_balance: HashMap<char, i32>,
}

impl Dispatcher {
    fn calculate_cargo_balance(&mut self, slots: &[Slot]) {
        self.cargo_balance.clear();

        slots.iter().for_each(|x| {
            let payloads = x.get_payloads();
            if let Some(payload) = payloads[0] {
                *self.cargo_balance.entry(payload.cargo).or_insert(0) += 1;
            }
            if let Some(payload) = payloads[1] {
                *self.cargo_balance.entry(payload.cargo).or_insert(0) -= 1;
            }
        });
        self.cargo_balance.retain(|_, v| *v != 0);
    }

    pub(crate) fn precalc(&mut self, slots: &[Slot]) {
        self.calculate_cargo_balance(slots);
    }

    pub(crate) fn conduct(&mut self, carriers: &mut Vec<Carrier>, slots: &mut Vec<Slot>) {
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
                                x.target_slot(ii, &mut slots[ii], false, false);
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
                            x.target_slot(slot_index, &mut slots[slot_index], false, true);
                            x.reserved_target = Some(pit_index);
                            self.reduce_cargo_balance(
                                slots[slot_index].get_payloads()[0].unwrap().cargo,
                            );
                        }
                    } else if let (Some(slot_index), possible_target) =
                        self.find_slot_with_mismatched_payload_and_free_target(slots)
                    {
                        x.target_slot(slot_index, &mut slots[slot_index], false, false);
                        slots[possible_target].taken_care_of = true;
                        x.reserved_target = Some(possible_target);
                    } else if let Some(slot_index) = self.find_slot_with_mismatched_payload(slots) {
                        x.target_slot(slot_index, &mut slots[slot_index], false, false);
                    }
                }
                State::LOOKINGFORTARGET => match x.reserved_target {
                    Some(slot_index) => x.target_slot(
                        slot_index,
                        &mut slots[slot_index],
                        x.temporary_target,
                        x.carrying_to_pit,
                    ),
                    None => match self.find_slot_for_target(slots, x.payload) {
                        Some(slot_index) => x.target_slot(
                            slot_index,
                            &mut slots[slot_index],
                            x.temporary_target,
                            x.carrying_to_pit,
                        ),
                        None => {
                            x.state = State::NOTARGET;
                        }
                    },
                },
                State::NOTARGET => match self.find_temporary_slot(slots, x.payload) {
                    Some(slot_index) => {
                        x.target_slot(slot_index, &mut slots[slot_index], true, false);
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

    fn reduce_cargo_balance(&mut self, cargo: char) {
        *self.cargo_balance.entry(cargo).or_insert(0) -= 1;
        self.cargo_balance.retain(|_, v| *v != 0);
    }

    fn find_pit(&self, slots: &[Slot]) -> Option<usize> {
        for (i, v) in slots.iter().enumerate() {
            if v.is_pit() {
                return Some(i);
            }
        }
        None
    }

    fn find_slot_with_payload_that_should_go_to_the_pit(&self, slots: &[Slot]) -> Option<usize> {
        let excessive = self.cargo_balance.iter().find(|&(_, v)| *v > 0);
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
    fn find_slot_that_contains(&self, slots: &[Slot], cargo: char) -> Option<usize> {
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

    fn is_there_a_free_slot_for(&self, payload: Payload, slots: &[Slot], ii: &mut usize) -> bool {
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
        slots: &[Slot],
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

    fn find_slot_with_mismatched_payload(&self, slots: &[Slot]) -> Option<usize> {
        slots.iter().position(|x| {
            let [current, target] = x.get_payloads();
            current != None && current != target && !x.taken_care_of
        })
    }

    fn find_slot_for_target(
        &self,
        slots: &[Slot],
        target_payload: Option<Payload>,
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

    fn find_temporary_slot(&self, slots: &[Slot], target: Option<Payload>) -> Option<usize> {
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
}
