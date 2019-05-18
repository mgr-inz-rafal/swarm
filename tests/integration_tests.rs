#[macro_use(make_carrier, make_slot)]
extern crate swarm;
use swarm::{Carrier, Payload, Slot, State};

#[test]
fn conduct_to_targetting() {
    let mut game = swarm::new();

    game.add_carrier(make_carrier!(0.0, 0.0));
    game.add_carrier(make_carrier!(0.0, 0.0));
    game.add_slot(make_slot!(
        100.0,
        100.0,
        Some(Payload::from_char('X')),
        None
    ));

    game.tick();

    // Carrier should have target set to slot with 'X'
    let state = game.get_carriers()[0].get_state();
    if let State::TARGETING(target) = state {
        assert_eq!(
            game.get_slots()[target].get_payloads()[0],
            Some(Payload::from_char('X'))
        )
    } else {
        panic!("Found Carrier that is 'targetting' but has no target set")
    }
}
