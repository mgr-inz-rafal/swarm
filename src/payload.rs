// TODO: type of cargo must be injected by the external caller and not hardcoded to 'char'
#[derive(Copy, Clone, Debug)]
pub struct Payload {
    pub cargo: char,
    pub(crate) taken_from: Option<usize>,
}

impl PartialEq for Payload {
    fn eq(&self, other: &Payload) -> bool {
        self.cargo == other.cargo
    }
}

impl Payload {
    pub fn from_char(c: char) -> Payload {
        Payload {
            cargo: c,
            taken_from: None,
        }
    }
}
