use crate::Position;

#[derive(Copy, Clone)]
pub struct Carrier {
    pos: Position,
}

impl Carrier {
    pub fn new(x: f64, y: f64) -> Carrier {
        Carrier {pos: Position::new(x, y)}
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }
}
