use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use chip8_core::core;
use chip8_core::peripherals::DownTimer;
use chip8_tools::util::load_program;
use chip8_tools::util::minifb::MinifbDisplay;
use chip8_core::Chip8;
use log::{debug, error, info};

const HELP: &str = "\
chip8-emu - An emulator for the CHIP-8 CPU

USAGE:
    chip8-emu ROM_FILE

ARGS:
    ROM_FILE    Path to a CHIP-8 ROM (*.ch8)
";

fn main() -> Result<()> {
    env_logger::init();

    let path = match std::env::args().skip(1).next() {
        Some(path) => path,
        None => {
            eprintln!("{}", HELP);
            return Ok(());
        }
    };

    let mut mem = vec![0; 4096];
    let mut reg = vec![0; 16];
    let mut stack = vec![0; 16];

    info!("Loading program from {}", path);
    load_program(&path, &mut mem[..]).with_context({
        let path = path.clone();
        move || format!("Loading program \"{}\"", path)
    })?;

    let mut minifb = MinifbDisplay::new(60).with_context(|| "Creating minifb display")?;
    let graphics_adapter = minifb.graphics_adapter();
    let keypad_adapter = minifb.keypad_adater();

    let (tx_stop_gui, rx_stop_gui) = channel();

    debug!("Spawning CHIP-8 thread");
    std::thread::spawn(move || {
        let mut chip8 = Chip8::new(
            core::Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
            700,
            keypad_adapter,
            graphics_adapter,
            DownTimer::new("delay"),
            DownTimer::new("sound"),
        );

        if let Err(e) = chip8.run() {
            error!("CHIP-8 stopped: {}", e);
            tx_stop_gui.send(()).expect("Sending stop to gui");
        }
    });

    debug!("Starting GUI");
    minifb.run(rx_stop_gui).with_context(|| "Running minifb")?;

    info!("Exiting");
    Ok(())
}
