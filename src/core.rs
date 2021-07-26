pub mod instructions;

use crate::{
    peripherals::{Graphics, Keys, Timer},
};

#[derive(Debug)]
pub struct Core<'memory> {
    mem: &'memory mut [u8],
    reg: &'memory mut [u8],
    stack: &'memory mut [u16],
    i: u16,
    pc: u16,
    sp: u8,
    wait_for_keypress: bool,
}

impl<'memory> Core<'memory> {
    pub fn new(mem: &'memory mut [u8], reg: &'memory mut [u8], stack: &'memory mut [u16]) -> Self {
        assert!(mem.len() >= 2048);
        assert!(reg.len() >= 16);
        assert!(stack.len() >= 16);

        Self {
            mem,
            reg,
            stack,
            i: 0,
            pc: 0,
            sp: 0,
            wait_for_keypress: false,
        }
    }

    pub fn tick<G, TD, TS>(
        &mut self,
        keys: Keys,
        graphics: &mut G,
        timer_delay: &mut TD,
        timer_sound: &mut TS,
    ) where
        G: Graphics,
        TD: Timer,
        TS: Timer,
    {
        match (self.wait_for_keypress, keys.pressed()) {
            (true, false) => return,
            (true, true) => self.wait_for_keypress = false,
            _ => (),
        }
    }
}
