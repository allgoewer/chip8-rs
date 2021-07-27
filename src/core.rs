pub mod instructions;

use crate::peripherals::{Graphics, Keys, Pos, Sprite, Timer};
use crate::Error;
use instructions::Instruction;
use std::convert::TryFrom;

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
    const VF: usize = 15;

    pub fn new(mem: &'memory mut [u8], reg: &'memory mut [u8], stack: &'memory mut [u16]) -> Self {
        assert!(mem.len() >= 2048);
        assert!(reg.len() >= 16);
        assert!(stack.len() >= 16);

        Self {
            mem,
            reg,
            stack,
            i: 0,
            pc: 0x200,
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
    ) -> Result<(), Error>
    where
        G: Graphics,
        TD: Timer,
        TS: Timer,
    {
        use instructions::Instruction::*;

        match (self.wait_for_keypress, keys.pressed()) {
            (true, false) => return Ok(()),
            (true, true) => self.wait_for_keypress = false,
            _ => (),
        }

        match Instruction::try_from(&self.mem[self.pc as usize..])? {
            // Clear the display
            I00E0 => {
                graphics.clear();
                graphics.refresh();
            }

            // Jump to address NNN
            I1NNN(addr) => self.pc = addr.0 - 2,

            // Set register to value
            I6XNN(reg, val) => self.reg[reg.0 as usize] = val.0,

            // Set register to register + value
            I7XNN(reg, val) => {
                let (result, _) = self.reg[reg.0 as usize].overflowing_add(val.0);
                self.reg[reg.0 as usize] = result;
            }

            // Set I to addr
            IANNN(addr) => self.i = addr.0,

            // Display sprite (length: val bytes) starting at memory location I at (reg0, reg1)
            // Set VF to 1 if collistion is detected
            IDXYN(reg0, reg1, val) => {
                let start_address = self.i as usize;
                let length = val.0 as usize;
                let reg0_value = self.reg[reg0.0 as usize];
                let reg1_value = self.reg[reg1.0 as usize];

                let pos = Pos(reg0_value, reg1_value);
                let sprite = Sprite(&self.mem[start_address..(start_address + length)]);

                self.reg[Self::VF] = if graphics.toggle_sprite(pos, sprite) {
                    1
                } else {
                    0
                };
                graphics.refresh();
            }

            // Unimplemented instructions
            _ => (),
        }

        // Increase the program counter
        self.pc += 2;

        Ok(())
    }
}
