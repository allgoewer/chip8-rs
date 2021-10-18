use chip8::core;
use chip8::peripherals::{DownTimer, NullKeypad};
use chip8::util::load_program;
use chip8::util::minifb::MinifbDisplay;
use chip8::Chip8;
use std::io::Write;
use std::sync::mpsc::channel;

fn main() {
    let path = std::env::args().skip(1).next().expect("Give ROM path");

    let mut mem = vec![0; 2048];
    let mut reg = vec![0; 16];
    let mut stack = vec![0; 16];

    load_program(path, &mut mem[..]).expect("Failed loading ROM");

    let mut minifb = MinifbDisplay::new(60).expect("Could not crate minifb display");
    let graphics_adapter = minifb.graphics_adapter();

    let (tx_exit_gui, rx_exit_gui) = channel();

    std::thread::spawn(move || {
        let mut chip8 = Chip8::new(
            core::Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
            700,
            NullKeypad,
            graphics_adapter,
            DownTimer::new("delay"),
            DownTimer::new("sound"),
        );

        println!("CHIP-8 Debugger");

        loop {
            let mut cmd = String::new();

            print!("cmd: ");
            std::io::stdout().flush().expect("couldn't flush stdout");

            if let Ok(_) = std::io::stdin().read_line(&mut cmd) {
                match &cmd[..] {
                    "\n" | "s\n" | "step\n" => {
                        chip8.tick().expect("Error ticking chip8");
                        println!("{}", chip8);
                        println!("");
                    }
                    "e\n" | "q\n" | "exit\n" | "quit\n" => break,
                    _ => (),
                }
            }
        }

        tx_exit_gui.send(()).expect("Sending exit to gui");
    });

    minifb.run(rx_exit_gui).expect("Running minifb failed");
}
