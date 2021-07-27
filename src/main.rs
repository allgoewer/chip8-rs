use chip8::core;
use chip8::peripherals::{DownTimer, Graphics, Pos, Sprite, NullGraphics, NullKeypad};
use chip8::Chip8;
use chip8::util::minifb::MinifbDisplay;

fn main() {
    let mut mem = [0; 2048];
    let mut reg = [0; 16];
    let mut stack = [0; 16];

    let mut minifb = MinifbDisplay::new(60).expect("Could not crate minifb display");

    let mut chip8 = Chip8::new(
        core::Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
        700,
        NullKeypad,
        NullGraphics,
        DownTimer::default(),
        DownTimer::default(),
    );

    let mut adapter = minifb.graphics_adapter();

    std::thread::spawn(move || {
        let sprite_a: [u8; 5] = [0xF0, 0x90, 0xF0, 0x90, 0x90];
        let sprite_b: [u8; 5] = [0xE0, 0x90, 0xE0, 0x90, 0xE0];
        let sprite_c: [u8; 5] = [0xF0, 0x80, 0x80, 0x80, 0xF0];
        let sprite_d: [u8; 5] = [0xE0, 0x90, 0x90, 0x90, 0xE0];

        loop {
            adapter.clear();
            adapter.refresh();
            std::thread::sleep(std::time::Duration::from_micros(1_000_000 / 1));
        }
    });

    minifb.run().expect("Running minifb failed");
}
