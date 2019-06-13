#[macro_use]
mod macros;

mod carrier;
mod dispatcher;
mod payload;
mod position;
mod slot;

use std::collections::HashMap;
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

#[derive(Default)]
pub struct Swarm<T: PartialEq + Eq + Hash + Copy> {
    carriers: Vec<Carrier<T>>,
    slots: Vec<Slot<T>>,
    first_tick: bool,
    idle_ticks: u8,
    dispatcher: Dispatcher<T>,
}

impl<T: PartialEq + Eq + Hash + Copy> Swarm<T> {
    pub fn new() -> Swarm<T> {
        Swarm {
            carriers: Vec::new(),
            slots: Vec::new(),
            first_tick: true,
            idle_ticks: 0,
            dispatcher: Dispatcher {
                cargo_balance: HashMap::new(),
            },
        }
    }

    fn add_object<U>(vec: &mut Vec<U>, obj: U) {
        vec.push(obj);
    }

    pub fn add_carrier(&mut self, carrier: Carrier<T>) {
        Swarm::<T>::add_object(&mut self.carriers, carrier);
    }

    pub fn add_slot(&mut self, slot: Slot<T>) {
        Swarm::<T>::add_object(&mut self.slots, slot);
    }

    pub fn get_carriers(&self) -> &Vec<Carrier<T>> {
        &self.carriers
    }

    pub fn get_slots(&self) -> &Vec<Slot<T>> {
        &self.slots
    }

    pub fn get_slots_mut(&mut self) -> &mut Vec<Slot<T>> {
        &mut self.slots
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

    pub fn tick(&mut self) -> bool {
        let mut slots = &mut self.slots;
        if self.first_tick {
            self.dispatcher.precalc(&slots);
            self.first_tick = false;
        }
        self.dispatcher.conduct(&mut self.carriers, &mut slots);
        self.carriers.iter_mut().for_each(|x| x.tick(slots));
        self.job_finished()
    }

    pub fn slot_data_changed(&mut self) {
        self.dispatcher.precalc(&self.slots);
    }
}

#[cfg(test)]
mod tests {}
