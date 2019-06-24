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
    /// ```
    /// let empty_slot = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// let slot_with_current_payload = swarm_it::Slot::<char>::new(
    ///     100.0, 100.0,
    ///     Some(swarm_it::Payload::new('X')),
    ///     None,
    ///     swarm_it::SlotKind::CLASSIC);
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

    /// Returns current slot position
    ///
    /// # Example
    ///
    /// ```
    /// let x = 100.0;
    /// let y = 200.0;
    /// let slot = swarm_it::Slot::<char>::new(x, y, None, None, swarm_it::SlotKind::CLASSIC);
    /// let position = slot.get_position();
    /// assert!(approx::relative_eq!(position.x, x));
    /// assert!(approx::relative_eq!(position.y, y));
    /// ```
    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Returns current carrier payloads.
    ///
    /// Index | Content
    /// ------|---------------
    /// 0     | current payload
    /// 1     | target payload
    ///
    /// # Example
    ///
    /// ```
    /// let slot_with_current_payload = swarm_it::Slot::<char>::new(
    ///     100.0, 100.0,
    ///     Some(swarm_it::Payload::new('X')),
    ///     None,
    ///     swarm_it::SlotKind::CLASSIC);
    /// let payloads = slot_with_current_payload.get_payloads();
    /// assert_eq!(payloads[0], Some(swarm_it::Payload::new('X')));
    /// assert_eq!(payloads[1], None);
    /// ```
    pub fn get_payloads(&self) -> [Option<Payload<T>>; 2] {
        [self.current_payload, self.target_payload]
    }

    /// Sets target payload
    ///
    /// # Example
    ///
    /// ```
    /// let mut empty_slot = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// empty_slot.set_target_payload(Some(swarm_it::Payload::new('X')));
    /// let payloads = empty_slot.get_payloads();
    /// assert_eq!(payloads[0], None);
    /// assert_eq!(payloads[1], Some(swarm_it::Payload::new('X')));
    /// ```
    pub fn set_target_payload(&mut self, p: Option<Payload<T>>) {
        self.target_payload = p;
    }

    /// Sets both current and target payloads
    ///
    /// # Example
    ///
    /// ```
    /// let mut empty_slot = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// empty_slot.set_payloads(Some(swarm_it::Payload::new('X')));
    /// let payloads = empty_slot.get_payloads();
    /// assert_eq!(payloads[0], Some(swarm_it::Payload::new('X')));
    /// assert_eq!(payloads[1], Some(swarm_it::Payload::new('X')));
    /// ```
    pub fn set_payloads(&mut self, p: Option<Payload<T>>) {
        self.current_payload = p;
        self.target_payload = p;
    }

    /// Returns `true` if this slot is already addressed by any of the carriers.
    /// It is mainly used by the library internals, but is also exposed
    /// for the user, so it is possible to, for example, prepare different
    /// visualization for these kind of slots.
    ///
    /// # Example
    ///
    /// ```
    /// let slot = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// assert_eq!(slot.is_taken_care_of(), false);
    /// ```
    pub fn is_taken_care_of(&self) -> bool {
        self.taken_care_of
    }

    /// Returns `true` is slot is a pit
    ///
    /// # Example
    ///
    /// ```
    /// let slot_classic = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// let slot_pit = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::PIT);
    /// let slot_spawner = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::SPAWNER);
    /// assert!(slot_pit.is_pit());
    /// assert!(!slot_classic.is_pit());
    /// assert!(!slot_spawner.is_pit());
    /// ```
    pub fn is_pit(&self) -> bool {
        self.kind == SlotKind::PIT
    }

    /// Returns `true` is slot is a spawner
    ///
    /// # Example
    ///
    /// ```
    /// let slot_classic = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::CLASSIC);
    /// let slot_pit = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::PIT);
    /// let slot_spawner = swarm_it::Slot::<char>::new(100.0, 100.0, None, None, swarm_it::SlotKind::SPAWNER);
    /// assert!(!slot_pit.is_spawner());
    /// assert!(!slot_classic.is_spawner());
    /// assert!(slot_spawner.is_spawner());
    /// ```
    pub fn is_spawner(&self) -> bool {
        self.kind == SlotKind::SPAWNER
    }

    pub(crate) fn accepts(&self, p: Option<Payload<T>>) -> bool {
        self.target_payload == p
    }
}
