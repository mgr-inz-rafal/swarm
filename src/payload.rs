#[derive(Copy, Clone, Debug)]
pub struct Payload<T: PartialEq> {
    pub cargo: T,
    pub taken_from: Option<usize>, // TODO: Make pub(crate) and allow consructing from T
}

impl<T: PartialEq> PartialEq for Payload<T> {
    fn eq(&self, other: &Payload<T>) -> bool {
        self.cargo == other.cargo
    }
}
