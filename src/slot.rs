use super::payload::*;
use super::position::*;

#[derive(Copy, Clone)]
pub struct Slot {
    pos: Position,
    pub(crate) current_payload: Option<Payload>,
    pub(crate) target_payload: Option<Payload>,
    pub(crate) taken_care_of: bool,
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
