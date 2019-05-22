#[macro_export]
macro_rules! make_carrier {
    ($x: expr, $y: expr) => {
        crate::Carrier::new($x, $y)
    };
}

#[macro_export]
macro_rules! make_slot {
    ($x: expr, $y: expr, $cp: expr, $tp: expr) => {
        crate::Slot::new($x, $y, $cp, $tp, crate::SlotKind::CLASSIC)
    };
}

#[macro_export]
macro_rules! make_slot_pit {
    ($x: expr, $y: expr) => {
        crate::Slot::new($x, $y, None, None, crate::SlotKind::PIT)
    };
}

#[macro_export]
macro_rules! make_slot_spawner {
    ($x: expr, $y: expr) => {
        crate::Slot::new($x, $y, None, None, crate::SlotKind::SPAWNER)
    };
}
