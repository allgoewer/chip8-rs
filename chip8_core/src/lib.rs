#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

//! A CHIP-8 emulator written in rust
//!
//! # Crate-level features
//! There is no `default` feature in this crate, stdlib support must be enabled manually.
//!
//! `std` : Enables stdlib support, by default the crate is compiled with `no_std`

/// The core CHIP-8 architecture
pub mod core;
/// The CHIP-8 instruction set
pub mod instructions;
/// The CHIP-8 peripherals. This consists of traits and default implementations.
pub mod peripherals;

pub use crate::core::Core;

use crate::peripherals::{Graphics, Keypad, Random, Timer};

/// Crate Error structure
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid instruction was encountered
    InvalidInstruction(u16),
    /// The decoded instruction has invalid alignemnt
    InvalidAlignment,
    /// A stack overflow occured during execution
    StackOverflow,
}

impl From<::core::array::TryFromSliceError> for Error {
    fn from(_: ::core::array::TryFromSliceError) -> Self {
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

/// A runnable CHIP-8 implementation. This includes a core + all necessary peripherals.
#[derive(Debug)]
pub struct Chip8<'memory, K, G, R, TD, TS> {
    core: Core<'memory>,
    core_freq: u32,
    keypad: K,
    graphics: G,
    random: R,
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

impl<'memory, K, G, R, TD, TS> Chip8<'memory, K, G, R, TD, TS>
where
    K: Keypad,
    G: Graphics,
    TD: Timer,
    TS: Timer,
    R: Random,
{
    /// Generate a new Chip8
    pub fn new(
        core: Core<'memory>,
        core_freq: u32,
        keypad: K,
        graphics: G,
        random: R,
        timer_delay: TD,
        timer_sound: TS,
    ) -> Self {
        Self {
            core,
            core_freq,
            keypad,
            graphics,
            random,
            timer_delay,
            timer_sound,
            timer_freq_div: core_freq / 60,
            timer_freq_count: 0,
        }
    }

    /// Run the Chip8
    ///
    /// Only available with the "std" feature, as [`std::thread::sleep`] is required.
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

    /// Execute a single tick of the Chip8
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
            &mut self.random,
            &mut self.timer_delay,
            &mut self.timer_sound,
        )
    }

    fn tick_timers(&mut self) {
        self.timer_delay.tick();
        self.timer_sound.tick();
    }
}
