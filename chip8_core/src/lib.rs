#![forbid(unsafe_code)]
#![warn(/*missing_docs,*/ missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod instructions;
pub mod peripherals;

use crate::peripherals::{FallingEdges, Graphics, Keypad, Keys, Pos, Sprite, Timer};
use instructions::{Instruction, Register};
#[cfg(feature = "std")]
use log::{debug, trace};

fn bcd(val: u8) -> (u8, u8, u8) {
    let hundreds = val / 100;

    let val = val - (hundreds * 100);
    let tens = val / 10;

    let ones = val - (tens * 10);

    (hundreds, tens, ones)
}

#[derive(Debug)]
pub struct Core<'memory, R> {
    mem: &'memory mut [u8],
    reg: &'memory mut [u8],
    stack: &'memory mut [u16],
    i: u16,
    pc: u16,
    sp: u8,
    last_instruction: Option<Instruction>,
    random_gen: R,
}

#[cfg(feature = "std")]
impl<R> std::fmt::Display for Core<'_, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(instruction) = &self.last_instruction {
            write!(
                f,
                "PC {:04X} SP {:02X} I {:04X} regs {:02X?} [{}]",
                self.pc, self.sp, self.i, self.reg, instruction
            )
        } else {
            write!(
                f,
                "PC {:04X} SP {:02X} I {:04X} regs {:02X?}",
                self.pc, self.sp, self.i, self.reg
            )
        }
    }
}

impl<'memory, R> Core<'memory, R>
where
    R: FnMut() -> u8,
{
    const VF: Register = Register(15);
    const FONT_LEN: usize = 5;

    pub fn new(mem: &'memory mut [u8], reg: &'memory mut [u8], stack: &'memory mut [u16], random_gen: R) -> Self {
        assert!(mem.len() >= 2048);
        assert!(reg.len() >= 16);
        assert!(stack.len() >= 16);

        Self::load_font(mem);

        Self {
            mem,
            reg,
            stack,
            i: 0,
            pc: 0x200,
            sp: 0,
            last_instruction: None,
            random_gen,
        }
    }

    fn load_font(loc: &mut [u8]) {
        loc[0..(Self::FONT_LEN * 16)].copy_from_slice(&[
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]);
    }

    pub fn tick<G, TD, TS>(
        &mut self,
        keys: Keys,
        mut edges: FallingEdges,
        graphics: &mut G,
        timer_delay: &mut TD,
        timer_sound: &mut TS,
    ) -> Result<(), Error>
    where
        G: Graphics,
        TD: Timer,
        TS: Timer,
    {
        enum ModPc {
            Hold,
            Normal,
            Skip(u16),
            Jump(u16),
            Ret(u16),
        }

        use instructions::Instruction::*;
        use ModPc::*;

        let mut pc_after = Normal;
        let mut pc = |pc| pc_after = pc;

        let instruction = Instruction::try_from(&self.mem[self.pc as usize..])?;
        match instruction.clone() {
            // SYS addr
            // Jump to a machine code routine at nnn
            I0NNN(_nnn) => unimplemented!(),

            // CLS
            // Clear the display
            I00E0 => {
                graphics.clear();
                graphics.refresh();
            }

            // RET
            // Return from a subroutine
            I00EE => pc(Ret(self.pop()?)),

            // JP addr
            // Jump to location nnn
            I1NNN(nnn) => pc(Jump(nnn.0)),

            // CALL addr
            // Call subroutine at nnn
            I2NNN(nnn) => {
                self.push(self.pc)?;
                pc(Jump(nnn.0));
            }

            // SE Vx, byte
            // Skip next instruction if Vx = kk
            I3XNN(x, vv) => {
                if *self.r(x) == vv.0 {
                    pc(Skip(1));
                }
            }

            // SNE Vx, byte
            // Skip next instruction if Vx != kk
            I4XNN(x, vv) => {
                if *self.r(x) != vv.0 {
                    pc(Skip(1));
                }
            }

            // SE Vx, Vy
            // Skip next instruction if Vx = Vy
            I5XY0(x, y) => {
                if *self.r(x) == *self.r(y) {
                    pc(Skip(1));
                }
            }

            // LD Vx, byte
            // Set Vx = kk
            I6XNN(x, vv) => *self.r(x) = vv.0,

            // Add Vx, byte
            // Set Vx = Vx + kk
            I7XNN(x, vv) => {
                let (val, _) = self.r(x.clone()).overflowing_add(vv.0);
                *self.r(x) = val;
            }

            // LD Vx, Vy
            // Set Vx = Vy
            I8XY0(x, y) => *self.r(x) = *self.r(y),

            // OR Vx, Vy
            // Set Vx = Vx OR Vy
            I8XY1(x, y) => *self.r(x) |= *self.r(y),

            // AND Vx, Vy
            // Set Vx = Vx AND Vy
            I8XY2(x, y) => *self.r(x) &= *self.r(y),

            // XOR Vx, Vy
            // Set Vx = Vx XOR Vy
            I8XY3(x, y) => *self.r(x) ^= *self.r(y),

            // ADD Vx, Vy
            // Set Vx = Vx + Vy, set VF = carry
            I8XY4(x, y) => {
                let (val, carry) = self.r(x.clone()).overflowing_add(*self.r(y));
                *self.r(x) = val;
                *self.r(Self::VF) = if carry { 1 } else { 0 };
            }

            // SUB Vx, Vy
            // Set Vx = Vx - Vy, set VF = NOT borrow
            I8XY5(x, y) => {
                let (val, carry) = self.r(x.clone()).overflowing_sub(*self.r(y));
                *self.r(x) = val;
                *self.r(Self::VF) = if carry { 0 } else { 1 };
            }

            // SHR Vx {, Vy}, set VF
            // Set Vx = Vx SHR 1
            I8XY6(x, _y) => {
                *self.r(Self::VF) = *self.r(x.clone()) & 0x01;
                *self.r(x) /= 2;
            }

            // SUBN Vy, Vx
            // Set Vx = Vy - Vx, set VF = NOT borrow
            I8XY7(x, y) => {
                let (val, carry) = self.r(y).overflowing_sub(*self.r(x.clone()));
                *self.r(x) = val;
                *self.r(Self::VF) = if carry { 0 } else { 1 };
            }

            // SHL Vx {, Vy}, set VF
            // Set Vx SHL 1
            I8XYE(x, _y) => {
                let (val, carry) = self.r(x.clone()).overflowing_mul(2);
                *self.r(x) = val;
                *self.r(Self::VF) = if carry { 1 } else { 0 };
            }

            // SNE Vx, Vy
            // Skip next instruction if Vx != Vy
            I9XY0(x, y) => {
                if *self.r(x) != *self.r(y) {
                    pc(Skip(1));
                }
            }

            // LD I, addr
            // Set I = addr
            IANNN(nnn) => self.i = nnn.0,

            // JP V0, addr
            // Jump to location nnn + V0
            IBNNN(nnn) => pc(Jump(nnn.0 + *self.r(Register(0)) as u16)),

            // RND Vx, byte
            // Set Vx = random byte AND kk
            ICXNN(x, vv) => {
                *self.r(x) = (self.random_gen)() & vv.0;
            }

            // DRW Vx, Vy, nibble
            // Display sprite (length: val bytes) starting at memory location I at (reg0, reg1)
            // Set VF to 1 if collistion is detected
            IDXYN(x, y, v) => {
                let start_address = self.i as usize;
                let length = v.0 as usize;
                let reg0_value = self.reg[x.0 as usize];
                let reg1_value = self.reg[y.0 as usize];

                let pos = Pos(reg0_value, reg1_value);
                let sprite = Sprite(&self.mem[start_address..(start_address + length)]);

                *self.r(Self::VF) = if graphics.toggle_sprite(pos, sprite) {
                    1
                } else {
                    0
                };
                graphics.refresh();
            }

            // SKP Vx
            // Skip next instruction if key with the value of Vx is pressed
            IEX9E(x) => {
                if keys.pressed(*self.r(x)) {
                    pc(Skip(1));
                }
            }

            // SKNP Vx
            // Skip next instruction if key with the value of Vx is not pressed
            IEXA1(x) => {
                if !keys.pressed(*self.r(x)) {
                    pc(Skip(1));
                }
            }

            // LD Tx, DT
            // Set Vx = delay timer value
            IFX07(x) => {
                *self.r(x) = timer_delay.get();
            }

            // LD Vx, K
            // Wait for a key press, store the value of the key in Vx
            IFX0A(x) => {
                let old_edges = edges.clone();
                if let Some(idx) = edges.pop_next_idx() {
                    #[cfg(feature = "std")]
                    debug!("IFX0A {:?}", old_edges);
                    *self.r(x) = idx;
                } else {
                    pc(Hold);
                }
            }

            // LD DT, Vx
            // Set delay timer = Vx
            IFX15(x) => timer_delay.set(*self.r(x)),

            // LD ST, Vx
            // Set sound timer = Vx
            IFX18(x) => timer_sound.set(*self.r(x)),

            // ADD I, Vx
            // Set I = I + Vx
            IFX1E(x) => {
                let (val, _) = self.i.overflowing_add(*self.r(x) as u16);
                self.i = val;
            }

            // LD F, Vx
            // Set I = location of sprite for digit Vx
            IFX29(x) => self.i = *self.r(x) as u16 * Self::FONT_LEN as u16,

            // LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1 and I+2
            IFX33(x) => {
                let (hundreds, tens, ones) = bcd(*self.r(x));
                self.mem[self.i as usize] = hundreds;
                self.mem[self.i as usize + 1] = tens;
                self.mem[self.i as usize + 2] = ones;
            }

            // LD [I], Vx
            // Store registers V0 through Vx in memory starting at location I
            IFX55(x) => {
                for i in 0..(x.0 + 1) {
                    self.mem[self.i as usize + i as usize] = *self.r(i.into());
                }
            }

            // LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I
            IFX65(x) => {
                for i in 0..(x.0 + 1) {
                    *self.r(i.into()) = self.mem[self.i as usize + i as usize];
                }
            }
        }

        // Update the program counter
        match pc_after {
            // Stall the program counter
            ModPc::Hold => (),
            // Jump to the next 16 bit instruction
            ModPc::Normal => self.pc += 2,
            // Skip the next n instructions (+ jump to the next 16 bit instruction)
            ModPc::Skip(n) => self.pc += 2 * (n + 1),
            // Set the PC to a fixed value
            ModPc::Jump(pc) => self.pc = pc,
            // Return from call
            ModPc::Ret(pc) => self.pc = pc + 2,
        }

        self.last_instruction = Some(instruction);

        #[cfg(feature = "std")]
        trace!("{}", self);

        Ok(())
    }

    fn r(&mut self, reg: Register) -> &mut u8 {
        &mut self.reg[reg.0 as usize]
    }

    fn pop(&mut self) -> Result<u16, Error> {
        self.sp -= 1;
        let val = self
            .stack
            .get(self.sp as usize)
            .ok_or(Error::StackOverflow)?;

        Ok(*val)
    }

    fn push(&mut self, val: u16) -> Result<(), Error> {
        *self
            .stack
            .get_mut(self.sp as usize)
            .ok_or(Error::StackOverflow)? = val;
        self.sp += 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bcd() {
        assert_eq!(super::bcd(123), (1, 2, 3));
        assert_eq!(super::bcd(023), (0, 2, 3));
        assert_eq!(super::bcd(003), (0, 0, 3));
    }
}


#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInstruction(u16),
    InvalidAlignment,
    StackOverflow,
}

impl From<core::array::TryFromSliceError> for Error {
    fn from(_: core::array::TryFromSliceError) -> Self {
        Self::InvalidAlignment
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstruction(ins) => write!(f, "Invalid instruction: 0x{:02X}", ins),
            Self::InvalidAlignment => write!(f, "Invalid alignment"),
            Self::StackOverflow => write!(f, "Stack overflow"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Chip8<'memory, K, G, TD, TS, R> {
    core: Core<'memory, R>,
    core_freq: u32,
    keypad: K,
    graphics: G,
    timer_delay: TD,
    timer_sound: TS,
    timer_freq_div: u32,
    timer_freq_count: u32,
}

#[cfg(feature = "std")]
impl<K, G, TD, TS, R> std::fmt::Display for Chip8<'_, K, G, TD, TS, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.core)
    }
}

impl<'memory, K, G, TD, TS, R> Chip8<'memory, K, G, TD, TS, R>
where
    K: Keypad,
    G: Graphics,
    TD: Timer,
    TS: Timer,
    R: FnMut() -> u8,
{
    pub fn new(
        core: Core<'memory, R>,
        core_freq: u32,
        keypad: K,
        graphics: G,
        timer_delay: TD,
        timer_sound: TS,
    ) -> Self {
        Self {
            core,
            core_freq,
            keypad,
            graphics,
            timer_delay,
            timer_sound,
            timer_freq_div: core_freq / 60,
            timer_freq_count: 0,
        }
    }

    #[cfg(feature = "std")]
    pub fn run(&mut self) -> Result<(), Error> {
        use std::thread::sleep;
        use std::time::{Duration, Instant};

        let cycle_duration = Duration::from_micros(1_000_000 / self.core_freq as u64);

        loop {
            let before_tick = Instant::now();
            self.tick()?;

            if let Some(remaining) = cycle_duration.checked_sub(before_tick.elapsed()) {
                sleep(remaining);
            }
        }
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        self.tick_core()?;

        self.timer_freq_count += 1;
        if self.timer_freq_count >= self.timer_freq_div {
            self.timer_freq_count = 0;
            self.tick_timers();
        }

        Ok(())
    }

    fn tick_core(&mut self) -> Result<(), Error> {
        let keys = self.keypad.pressed_keys();
        let edges = self.keypad.last_released_key();

        self.core.tick(
            keys,
            edges,
            &mut self.graphics,
            &mut self.timer_delay,
            &mut self.timer_sound,
        )
    }

    fn tick_timers(&mut self) {
        self.timer_delay.tick();
        self.timer_sound.tick();
    }
}
