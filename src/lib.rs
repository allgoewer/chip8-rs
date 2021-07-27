pub mod core;
pub mod peripherals;
pub mod util;

use crate::core::Core;
use crate::peripherals::{Graphics, Keypad, Timer};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInstruction(u16),
    InvalidAlignment,
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(_: std::array::TryFromSliceError) -> Self {
        Self::InvalidAlignment
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstruction(ins) => write!(f, "Invalid instruction: 0x{:02X}", ins),
            Self::InvalidAlignment => write!(f, "Invalid alignment"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Chip8<'memory, K, G, TD, TS> {
    core: Core<'memory>,
    core_freq: u32,
    keypad: K,
    graphics: G,
    timer_delay: TD,
    timer_sound: TS,
    timer_freq_div: u32,
    timer_freq_count: u32,
}

impl<'memory, K, G, TD, TS> Chip8<'memory, K, G, TD, TS>
where
    K: Keypad,
    G: Graphics,
    TD: Timer,
    TS: Timer,
{
    pub fn new(
        core: Core<'memory>,
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

        Ok(())
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
        self.core.tick(
            keys,
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
