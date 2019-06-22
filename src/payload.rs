#[derive(Copy, Clone, Debug)]
/// Represnets the `Payload` object. Payload is transferred around by Carriers.
pub struct Payload<T: PartialEq> {
    /// This is the actual payload
    pub cargo: T,

    pub(crate) taken_from: Option<usize>,
}

impl<T: PartialEq> Payload<T> {
    /// Creates new Payload
    ///
    /// # Example
    ///
    /// ```
    /// let payload = swarm_it::Payload::<char>::new('X');
    /// ```
    pub fn new(cargo: T) -> Payload<T> {
        Payload {
            cargo,
            taken_from: None,
        }
    }
}

impl<T: PartialEq> PartialEq for Payload<T> {
    fn eq(&self, other: &Payload<T>) -> bool {
        self.cargo == other.cargo
    }
}
