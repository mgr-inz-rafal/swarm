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
