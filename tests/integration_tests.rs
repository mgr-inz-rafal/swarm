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

/*
#[test]
fn rotate_direction_calculation1() {
    let mut game = swarm::new();

    game.add_carrier(make_carrier!(0.0, 0.0));
    let mut carrier = game.get_carriers()[0];
    carrier.angle = 0.0;
    carrier.rotate_to(std::f64::consts::PI / 2.0);

    assert_eq!(
        carrier.rotation_direction.unwrap(),
        RotationDirection::CLOCKWISE
    )
}
*/

/*
#[test]
fn rotate_direction_calculation2() {
    let mut game = swarm::new();

    game.add_carrier(make_carrier!(0.0, 0.0));
    let mut carrier = game.get_carriers()[0];
    carrier.angle = 0.0;
    carrier.rotate_to(std::f64::consts::PI / 2.0 * 3.0);

    assert_eq!(
        carrier.rotation_direction.unwrap(),
        RotationDirection::COUNTERCLOCKWISE
    )
}
*/

/*
#[test]
fn rotate_direction_calculation3() {
    let mut game = swarm::new();

    game.add_carrier(make_carrier!(0.0, 0.0));
    let mut carrier = game.get_carriers()[0];
    carrier.angle = 0.0;
    carrier.rotate_to(std::f64::consts::PI);

    // When rotation 180deg, choose either left or right direction
    assert!(carrier.rotation_direction.is_some())
}

*/
