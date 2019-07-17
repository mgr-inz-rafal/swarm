use super::position::*;

pub(crate) fn distance_between_positions(p1: &Position, p2: &Position) -> f64 {
    ((p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y)).sqrt()
}
