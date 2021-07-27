use std::io::Read;

use chip8::core;
use chip8::peripherals::{DownTimer, Graphics, NullGraphics, NullKeypad, Pos, Sprite};
use chip8::util::minifb::MinifbDisplay;
use chip8::Chip8;

fn load_program<P: AsRef<std::path::Path>>(path: P, target: &mut [u8]) -> std::io::Result<()> {
    let mut rom = std::fs::File::open(path.as_ref())?;
    rom.read(&mut target[0x200..])?;

    Ok(())
}

fn main() {
    let mut mem = vec![0; 2048];
    let mut reg = vec![0; 16];
    let mut stack = vec![0; 16];

    load_program("roms/IBM Logo.ch8", &mut mem[..]).expect("Failed loading ROM");

    let mut minifb = MinifbDisplay::new(60).expect("Could not crate minifb display");
    let graphics_adapter = minifb.graphics_adapter();

    std::thread::spawn(move || {
        let mut chip8 = Chip8::new(
            core::Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
            700,
            NullKeypad,
            graphics_adapter,
            DownTimer::default(),
            DownTimer::default(),
        );

        if let Err(e) = chip8.run() {
            println!("CHIP-8 Error: {}", e);
        }
    });

    minifb.run().expect("Running minifb failed");
}
