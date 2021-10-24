use chip8_core::instructions::Instruction;
use chip8_core::Error;
use chip8_tools::util::load_program;

fn main() {
    let mut rom = vec![0; 2048];
    let path = std::env::args().nth(1).expect("Give path to ROM");

    load_program(path, &mut rom[..]).expect("Failed loading ROM");

    for (idx, chunk) in rom.chunks(2).skip(0x100).enumerate() {
        let addr = 0x200 + idx * 2;

        match Instruction::try_from(chunk) {
            Ok(opcode) => println!("0x{:04X}  {}", addr, opcode),
            Err(Error::InvalidInstruction(opcode)) => {
                println!("0x{:04X}               ; 0x{:04X} (invalid)", addr, opcode)
            }
            Err(e) => println!("0x{:04X}  {:<10}", addr, e),
        }
    }
}
