use crate::Position;

#[derive(Copy, Clone)]
pub struct Carrier {
    pos: Position,
    angle: f64,
}

impl Carrier {
    pub fn new(x: f64, y: f64) -> Carrier {
        Carrier {
            pos: Position::new(x, y),
            angle: 0.0,
        }
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    pub fn tick(&mut self) {
        self.pos.x = self.pos.x + self.angle.cos();
        self.pos.y = self.pos.y + self.angle.sin();
        self.angle += 0.01
    }
}
