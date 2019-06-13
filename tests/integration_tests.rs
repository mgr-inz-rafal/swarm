extern crate swarm;
use swarm::{Carrier, Payload, Slot, SlotKind, State};

#[test]
fn conduct_to_targetting() {
    let mut game = swarm::Swarm::new();

    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        Some(swarm::Payload::new('X')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        None,
        Some(swarm::Payload::new('X')),
        SlotKind::CLASSIC,
    ));

    game.tick();

    // Carrier should have target set to slot with 'X'
    let state = game.get_carriers()[0].get_state();
    if let State::TARGETING(target) = state {
        assert_eq!(
            game.get_slots()[target].get_payloads()[0],
            Some(Payload::new('X'))
        )
    } else {
        panic!("Found Carrier that is 'targetting' but has no target set")
    }
}

#[test]
fn idle_carriers_reporting() {
    let mut game = swarm::Swarm::new();

    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        Some(swarm::Payload::new('X')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        None,
        Some(swarm::Payload::new('X')),
        SlotKind::CLASSIC,
    ));

    let mut all_carriers_idle = game.tick();
    assert!(!all_carriers_idle);
    for _ in 1..100 {
        all_carriers_idle = game.tick();
    }
    assert!(all_carriers_idle);
}
