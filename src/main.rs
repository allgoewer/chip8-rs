use chip8::core;
use chip8::peripherals::{DownTimer, NullGraphics, NullKeypad};
use chip8::Chip8;

fn main() {
    let mut mem = [0; 2048];
    let mut reg = [0; 16];
    let mut stack = [0; 16];

    let mut chip8 = Chip8::new(
        core::Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
        60,
        NullKeypad,
        NullGraphics,
        DownTimer::default(),
        DownTimer::default(),
    );

    chip8.tick();
}
