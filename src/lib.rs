#[macro_use]
mod macros;

mod carrier;
mod dispatcher;
mod payload;
mod position;
mod slot;

use std::hash::Hash;

pub use carrier::*;
pub use dispatcher::*;
pub use payload::*;
pub use slot::*;

#[macro_use]
extern crate approx;

fn _debug_dump_slots(slots: &[Slot<char>]) {
    for (i, v) in slots.iter().enumerate() {
        print!("Slot [{}]: ", i);

        match slots[i].current_payload {
            Some(p) => print!("{} ", p.cargo),
            None => print!("None "),
        }

        let [_, target] = v.get_payloads();
        match target {
            Some(p) => print!("    {} ", p.cargo),
            None => print!("    None "),
        }

        print!("\tTaken care of={}", v.taken_care_of);

        println!();
    }
}

fn _debug_dump_slot_distances<T: PartialEq + Eq + Hash + Copy>(
    slots: &[Slot<T>],
    dispatcher: &Dispatcher<T>,
) {
    slots.iter().enumerate().for_each(|(i1, _)| {
        slots.iter().enumerate().for_each(|(i2, _)| {
            println!("{}->{} = {}", i1, i2, dispatcher.get_slot_distance(i1, i2));
        })
    });
}

// Main struct that allow you to talk to the library
#[derive(Default)]
pub struct Swarm<T: PartialEq + Eq + Hash + Copy> {
    carriers: Vec<Carrier<T>>,
    slots: Vec<Slot<T>>,
    first_tick: bool,
    idle_ticks: u8,
    dispatcher: Dispatcher<T>,
}

impl<T: PartialEq + Eq + Hash + Copy> Swarm<T> {
    /// Constructs a new `Swarm`.
    ///
    /// `T` - the type of cargo that your carriers will carry around. The `Copy` bound is going to be removed in future versions.
    ///
    /// # Example
    ///
    /// ```
    /// let game = swarm_it::Swarm::<char>::new();
    /// ```
    pub fn new() -> Swarm<T> {
        Swarm {
            carriers: Vec::new(),
            slots: Vec::new(),
            first_tick: true,
            idle_ticks: 0,
            dispatcher: Dispatcher::new(),
        }
    }

    /// Adds new carrier
    ///
    /// # Example
    ///
    /// ```
    /// use swarm_it::*;
    /// let mut game = Swarm::<char>::new();
    /// game.add_carrier(Carrier::new(100.0, 200.0));
    /// ```
    pub fn add_carrier(&mut self, carrier: Carrier<T>) {
        Swarm::<T>::add_object(&mut self.carriers, carrier);
    }

    /// Adds new slot
    ///
    /// # Example
    ///
    /// ```
    /// use swarm_it::*;
    /// let mut game = Swarm::<char>::new();
    /// game.add_slot(Slot::new(100.0, 100.0, None, Some(Payload::new('X')), swarm_it::SlotKind::CLASSIC));
    /// ```
    pub fn add_slot(&mut self, slot: Slot<T>) {
        Swarm::<T>::add_object(&mut self.slots, slot);
    }

    /// Returns all carriers
    pub fn get_carriers(&self) -> &Vec<Carrier<T>> {
        &self.carriers
    }

    /// Returns all slots
    pub fn get_slots(&self) -> &Vec<Slot<T>> {
        &self.slots
    }

    /// Returns all slots (To be deprecated)
    pub fn get_slots_mut(&mut self) -> &mut Vec<Slot<T>> {
        &mut self.slots
    }

    /// The engine must be regularly ticked by the outside world by invoking this function.
    /// At each tick swarm will perform calculation of the internal state logic, move
    /// carriers around, etc.
    ///
    /// Returns `true` if there were no more action required, meaning that carriers have finished
    /// tranferring the layout to target position.
    ///
    /// # Example
    ///
    /// ```
    /// use swarm_it::*;
    /// let mut game = Swarm::<char>::new();
    /// if game.tick() { println!("Job finished, yay!"); };
    /// ```
    pub fn tick(&mut self) -> bool {
        let mut slots = &mut self.slots;
        if self.first_tick {
            self.dispatcher.precalc(&slots);
            self.first_tick = false;
            //_debug_dump_slot_distances(&slots, &self.dispatcher);
        }
        self.dispatcher.conduct(&mut self.carriers, &mut slots);
        self.carriers.iter_mut().for_each(|x| x.tick(slots));
        self.job_finished()
    }

    /// Initiates some precalculation in order for the carriers
    /// to be aware of modified slots. Call this function each time you
    /// finished adding new slots or manually manipulating their data
    /// through `get_slots_mut()`.
    ///
    /// In the future I might eventually remove this obligation, but I need
    /// to come up with some clever way of deciding whether a precalc is
    /// needed or not. Invoking precalc after each single change is an overkill, since
    /// one might want to add 100k slots and do the precalc only once at the end.
    ///
    /// # Example
    ///
    /// ```
    /// use swarm_it::*;
    /// let mut game = Swarm::<char>::new();
    /// game.slot_data_changed();
    /// ```
    ///
    /// # TODO
    /// Different kind of that may change. Provide additional
    /// parameter in order NOT to precalculate everything
    /// 1. Slot payload => recalculate cargo balance
    /// 2. Slots added/removed => recalculate slot distances
    pub fn slot_data_changed(&mut self) {
        self.dispatcher.precalc(&self.slots);
    }

    fn add_object<U>(vec: &mut Vec<U>, obj: U) {
        vec.push(obj);
    }

    fn all_carriers_idle(&self) -> bool {
        !self.carriers.iter().any(|c| !c.state.is_idle())
    }

    fn job_finished(&mut self) -> bool {
        if self.all_carriers_idle() {
            self.idle_ticks += 1;
            if self.idle_ticks == std::u8::MAX {
                self.idle_ticks = 3;
            }
            if self.idle_ticks >= 2 {
                return true;
            }
        } else {
            self.idle_ticks = 0;
        }
        false
    }
}

#[cfg(test)]
mod tests {}
