pub mod core;
pub mod peripherals;
pub mod util;

use crate::core::Core;
use crate::peripherals::{Graphics, Keypad, Timer};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInstruction,
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
            Self::InvalidInstruction => write!(f, "Invalid instruction"),
            Self::InvalidAlignment => write!(f, "Invalid alignment"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Chip8<'memory, K, G, TD, TS> {
    core: Core<'memory>,
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
        freq: u32,
        keypad: K,
        graphics: G,
        timer_delay: TD,
        timer_sound: TS,
    ) -> Self {
        Self {
            core,
            keypad,
            graphics,
            timer_delay,
            timer_sound,
            timer_freq_div: freq / 60,
            timer_freq_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_core();

        self.timer_freq_count += 1;
        if self.timer_freq_count >= self.timer_freq_div {
            self.timer_freq_count = 0;
            self.tick_timers();
        }
    }

    fn tick_core(&mut self) {
        let keys = self.keypad.pressed_keys();
        self.core.tick(
            keys,
            &mut self.graphics,
            &mut self.timer_delay,
            &mut self.timer_sound,
        );
    }

    fn tick_timers(&mut self) {
        self.timer_delay.tick();
        self.timer_sound.tick();
    }
}
