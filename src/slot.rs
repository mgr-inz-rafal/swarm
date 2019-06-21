use super::payload::*;
use super::position::*;

use std::hash::Hash;

#[derive(Copy, Clone, PartialEq, Debug)]
/// Library supports different kind of slots
///
/// Type    | Meaning
/// --------|--------
/// CLASSIC | Slot with the ability to store single payload
/// PIT     | Slot which is always empty and carriers can drop anything into it (i.e. payload that is of no use and must be removed)
/// SPAWNER | Slot which produces any payload that might be needed by carriers to fill the payload shortage
pub enum SlotKind {
    CLASSIC,
    PIT,
    SPAWNER,
}

/// Represnets the `Slot` object. Slots have their target payload specified
/// and carriers will do their best to find appropriate payload and
/// bring it to the slot.
#[derive(Copy, Clone, Debug)]
pub struct Slot<T: PartialEq + Eq + Hash + Copy> {
    // TODO: Do not require Copy - get_payloads() should return reference
    pos: Position,
    pub(crate) current_payload: Option<Payload<T>>,
    target_payload: Option<Payload<T>>,
    pub(crate) taken_care_of: bool,
    kind: SlotKind,
}

impl<T: PartialEq + Eq + Hash + Copy> Slot<T> {
    /// Creates new Slot at the position specified and with the payloads specified
    ///
    /// # Example
    ///
    /// ```skip
    /// game.add_slot(Slot::new(100.0, 100.0, None, Some(Payload::new('X')), SlotKind::CLASSIC));
    /// ```
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

    /// Returns current carrier position
    ///
    /// # Example
    ///
    /// ```skip
    /// let position = slot.get_position();
    /// ```
    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Returns current carrier payloads.
    /// arr[0] - current payload
    /// arr[1] - target payload
    ///
    /// # Example
    ///
    /// ```skip
    /// let payloads = slot.get_payloads();
    /// ```
    pub fn get_payloads(&self) -> [Option<Payload<T>>; 2] {
        [self.current_payload, self.target_payload]
    }

    pub fn set_target_payload(&mut self, p: Option<Payload<T>>) {
        self.target_payload = p;
    }

    pub fn set_payloads(&mut self, p: Option<Payload<T>>) {
        self.current_payload = p;
        self.target_payload = p;
    }

    /// Returns `true` if this slot is already addressed by any of the carriers.
    /// It is mainly used by the library internals, but is also exposed
    /// for the user, so it is possible to, for example, prepare different
    /// visualization for these kind of slots.
    pub fn is_taken_care_of(&self) -> bool {
        self.taken_care_of
    }

    pub(crate) fn accepts(&self, p: Option<Payload<T>>) -> bool {
        self.target_payload == p
    }

    /// Returns `true` is slot is a pit
    pub fn is_pit(&self) -> bool {
        self.kind == SlotKind::PIT
    }

    /// Returns `true` is slot is a spawner
    pub fn is_spawner(&self) -> bool {
        self.kind == SlotKind::SPAWNER
    }
}
