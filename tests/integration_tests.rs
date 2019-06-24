#[macro_use(make_slot_pit, make_slot_spawner)]
extern crate swarm_it;
use swarm_it::{Carrier, Payload, Slot, SlotKind, State};

#[test]
fn conduct_to_targetting() {
    let mut game = swarm_it::Swarm::new();

    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        Some(swarm_it::Payload::new('X')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        None,
        Some(swarm_it::Payload::new('X')),
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
    let mut game = swarm_it::Swarm::new();

    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_carrier(Carrier::new(0.0, 0.0));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        Some(swarm_it::Payload::new('X')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        100.0,
        100.0,
        None,
        Some(swarm_it::Payload::new('X')),
        SlotKind::CLASSIC,
    ));

    let mut all_carriers_idle = game.tick();
    assert!(!all_carriers_idle);
    for _ in 1..100 {
        all_carriers_idle = game.tick();
    }
    assert!(all_carriers_idle);
}

#[test]
fn issue19_try_to_crash_lots_of_carriers() {
    // https://github.com/mgr-inz-rafal/swarm/issues/19

    fn increase_payload(p: &mut u32) {
        *p += 1;
        if *p > MAX_PAYLOAD {
            *p = 0
        };
    };

    let mut game = swarm_it::Swarm::new();

    // Add a lot of carriers
    for _ in 0..1000 {
        game.add_carrier(Carrier::new(0.0, 0.0));
    }

    // Add slots
    const MAX_PAYLOAD: u32 = 26;
    let mut current_payload = 0;
    let mut target_payload = (MAX_PAYLOAD - current_payload) / 2;
    for i in 0..10 {
        for j in 0..10 {
            game.add_slot(Slot::new(
                0.0 + f64::from(j) * 40.0,
                0.0 + f64::from(i) * 40.0,
                Some(Payload::new(current_payload)),
                Some(Payload::new(target_payload)),
                SlotKind::CLASSIC,
            ));
        }
        increase_payload(&mut current_payload);
        increase_payload(&mut target_payload);
    }

    // Add some empty slots
    for i in 0..5 {
        game.add_slot(Slot::new(
            15.0 + f64::from(i) * 35.0,
            15.0 + f64::from(i) * 35.0,
            None,
            None,
            SlotKind::CLASSIC,
        ));
    }

    // Add Pit and Spawner
    game.add_slot(make_slot_pit!(600.0, -50.0));
    game.add_slot(make_slot_spawner!(200.0, -50.0));

    // Get ready
    game.slot_data_changed();

    // Execute 1000 tics, expect no panic
    for _ in 0..1000 {
        game.tick();
    }
}
