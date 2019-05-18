use super::carrier::*;
use super::payload::*;
use super::slot::*;

pub struct Dispatcher {}
impl Dispatcher {
    pub(crate) fn conduct(carriers: &mut Vec<Carrier>, slots: &mut Vec<Slot>) {
        let mut _debug_carrier_indexer = 0;
        carriers.iter_mut().for_each(|x| {
            match x.state {
                State::MOVING(target) => {
                    if let Some(payload) = x.payload {
                        if x.temporary_target {
                            let mut ii: usize = 0;
                            let is_another_slot =
                                Dispatcher::is_there_a_free_slot_for(payload, slots, &mut ii);
                            if is_another_slot && ii != target {
                                x.target_slot(ii, &mut slots[ii], false);
                                slots[target].taken_care_of = false;
                            }
                        }
                    }
                }
                State::IDLE => {
                    if let (Some(slot_index), possible_target) =
                        Dispatcher::find_slot_with_mismatched_payload_and_free_target(slots)
                    {
                        x.target_slot(slot_index, &mut slots[slot_index], false);
                        slots[possible_target].taken_care_of = true;
                        x.reserved_target = Some(possible_target);
                    } else if let Some(slot_index) =
                        Dispatcher::find_slot_with_mismatched_payload(slots)
                    {
                        x.target_slot(slot_index, &mut slots[slot_index], false);
                    }
                }
                State::LOOKINGFORTARGET => match x.reserved_target {
                    Some(slot_index) => x.target_slot(slot_index, &mut slots[slot_index], false),
                    None => match Dispatcher::find_slot_for_target(slots, x.payload) {
                        Some(slot_index) => {
                            x.target_slot(slot_index, &mut slots[slot_index], false)
                        }
                        None => {
                            x.state = State::NOTARGET;
                        }
                    },
                },
                State::NOTARGET => match Dispatcher::find_temporary_slot(slots, x.payload) {
                    Some(slot_index) => {
                        x.target_slot(slot_index, &mut slots[slot_index], true);
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

#[cfg(test)]
mod tests {
    use crate::dispatcher::*;

    #[test]
    fn find_slot_for_target() {
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
            Dispatcher::find_slot_for_target(&slots, Some(Payload::from_char('B'))),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot1() {
        let slots = vec![
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('X'))),
            make_slot!(100.0, 100.0, None, Some(Payload::from_char('Y'))),
        ];

        // Slot without current payload cannot have mismatched payload
        assert_eq!(Dispatcher::find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot2() {
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
            Dispatcher::find_slot_with_mismatched_payload(&slots),
            Some(1)
        )
    }

    #[test]
    fn find_mismatched_slot3() {
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

        assert_eq!(Dispatcher::find_slot_with_mismatched_payload(&slots), None)
    }

    #[test]
    fn find_mismatched_slot_with_target1() {
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
            Dispatcher::find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target2() {
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
            Dispatcher::find_slot_with_mismatched_payload_and_free_target(&slots),
            (None, 0)
        )
    }

    #[test]
    fn find_mismatched_slot_with_target3() {
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
            Dispatcher::find_slot_with_mismatched_payload_and_free_target(&slots),
            (Some(0), 1)
        )
    }
}
