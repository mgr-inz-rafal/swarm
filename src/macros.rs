/// Helper macro for easier creation of `Pit` slots
#[macro_export]
macro_rules! make_slot_pit {
    ($x: expr, $y: expr) => {
        crate::Slot::new($x, $y, None, None, crate::SlotKind::PIT)
    };
}

/// Helper macro for easier creation of `Spawner` slots
#[macro_export]
macro_rules! make_slot_spawner {
    ($x: expr, $y: expr) => {
        crate::Slot::new($x, $y, None, None, crate::SlotKind::SPAWNER)
    };
}
