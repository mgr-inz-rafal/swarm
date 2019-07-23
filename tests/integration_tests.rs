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
fn parallel_transfer_to_pit() {
    let mut game = swarm_it::Swarm::new();

    game.add_carrier(Carrier::new(50.0, 50.0));
    game.add_carrier(Carrier::new(50.0, 50.0));
    game.add_slot(Slot::new(
        200.0,
        200.0,
        Some(Payload::new('A')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(Slot::new(
        300.0,
        250.0,
        Some(Payload::new('A')),
        None,
        SlotKind::CLASSIC,
    ));
    game.add_slot(make_slot_pit!(0.0, 0.0));

    game.tick();

    // Both carriers should target the pit
    let carriers = game.get_carriers();
    assert_eq!(carriers[0].get_reserved_target().unwrap(), 2);
    assert_eq!(carriers[1].get_reserved_target().unwrap(), 2);
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
    for _ in 0..60 {
        game.tick();
    }
}

#[test]
fn issue26_high_speed_carrier_not_reaching_target() {
    // https://github.com/mgr-inz-rafal/swarm/issues/26

    let mut game = swarm_it::Swarm::new();

    let i = game.add_carrier(Carrier::new(50.0, 50.0));
    game.get_carriers_mut()[i].set_acceleration(2.0);
    game.set_carrier_max_speed(i, 50.0);

    game.add_slot(Slot::new(
        200.0,
        200.0,
        Some(Payload::new('A')),
        Some(Payload::new('B')),
        SlotKind::CLASSIC,
    ));
    game.add_slot(make_slot_pit!(0.0, 0.0));

    // Get ready
    game.slot_data_changed();

    // After 60 ticks expect that carrier has a cargo
    for _ in 0..60 {
        game.tick();
    }

    let carriers = game.get_carriers();
    let payload = carriers[i].get_payload();
    assert!(payload.is_some());
}
