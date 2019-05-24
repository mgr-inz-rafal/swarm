use super::payload::*;
use super::position::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SlotKind {
    CLASSIC,
    PIT,
    SPAWNER,
}

#[derive(Copy, Clone, Debug)]
pub struct Slot<T: PartialEq + Copy> {
    // TODO: Do not require Copy - get_payloads() should return reference
    pos: Position,
    pub(crate) current_payload: Option<Payload<T>>,
    target_payload: Option<Payload<T>>,
    pub(crate) taken_care_of: bool,
    kind: SlotKind,
}

impl<T: PartialEq + Copy> Slot<T> {
    pub fn new(
        x: f64,
        y: f64,
        current_payload: Option<Payload<T>>,
        target_payload: Option<Payload<T>>,
        kind: SlotKind,
    ) -> Slot<T> {
        Slot {
            pos: Position::new(x, y),
            current_payload,
            target_payload,
            taken_care_of: false,
            kind,
        }
    }
    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_payloads(&self) -> [Option<Payload<T>>; 2] {
        [self.current_payload, self.target_payload]
    }

    pub fn is_taken_care_of(&self) -> bool {
        self.taken_care_of
    }

    pub(crate) fn accepts(&self, p: Option<Payload<T>>) -> bool {
        self.target_payload == p
    }

    pub fn is_pit(&self) -> bool {
        self.kind == SlotKind::PIT
    }

    pub fn is_spawner(&self) -> bool {
        self.kind == SlotKind::SPAWNER
    }
}
