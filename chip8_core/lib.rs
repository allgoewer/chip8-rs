use std::convert::TryFrom;

#[derive(Debug)]
pub enum Error {
    InvalidInstruction,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstruction => write!(f, "Invalid instruction"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Keys(u16);

impl Keys {
    fn pressed(&self) -> bool {
        self.0 != 0
    }
}

pub trait Keypad {
    fn pressed_keys(&self) -> Keys;
}

#[derive(Debug)]
pub struct NullKeypad;

impl Keypad for NullKeypad {
    fn pressed_keys(&self) -> Keys {
        Keys(0)
    }
}

pub trait Timer {
    fn tick(&mut self) -> bool;
    fn get(&self) -> u8;
    fn set(&mut self, val: u8);
}

#[derive(Debug)]
pub struct DownTimer(u8);

impl Timer for DownTimer {
    fn tick(&mut self) -> bool {
        let (new_val, overflow) = self.0.overflowing_sub(1);
        self.0 = new_val;
        
        overflow
    }
    
    fn get(&self) -> u8 {
        self.0
    }
    
    fn set(&mut self, val: u8) {
        self.0 = val;
    }
}

#[derive(Debug)]
pub struct Pos(u8, u8);

#[derive(Debug)]
pub struct Sprite<'memory>(&'memory [u8]);

pub trait Graphics {
    fn clear(&mut self);
    fn draw_sprite(&mut self, pos: Pos, sprite: Sprite<'_>);
}

#[derive(Debug)]
pub struct NullGraphics;

impl Graphics for NullGraphics {
    fn clear(&mut self) {}
    fn draw_sprite(&mut self, _pos: Pos, _sprite: Sprite<'_>) {}
}

#[derive(Debug)]
pub struct Register(u8);

impl From<u16> for Register {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}

#[derive(Debug)]
pub struct RegisterRange(u8);

impl From<u16> for RegisterRange {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}


#[derive(Debug)]
pub struct Address(u16);

impl From<u16> for Address {
    fn from(val: u16) -> Self {
        Self(val & 0x0FFF)
    }
}

#[derive(Debug)]
pub struct Value8(u8);

impl From<u16> for Value8 {
    fn from(val: u16) -> Self {
        Self((val & 0x00FF) as u8)
    }
}

#[derive(Debug)]
pub struct Value4(u8);

impl From<u16> for Value4 {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}

trait Nibbles {
    type Output;
    fn nibbles(&self) -> Self::Output;
}

impl Nibbles for u16 {
    type Output = (u16, u16, u16, u16);
    
    fn nibbles(&self) -> Self::Output {
        ((*self >> 24) & 0xF, (*self >> 16) & 0xF, (*self >> 8) & 0xF, *self & 0xF)
    }
}

#[derive(Debug)]
pub enum Instruction {
    I0NNN(Address),
    I00E0,
    I00EE,
    I1NNN(Address),
    I2NNN(Address),
    I3XNN(Register, Value8),
    I4XNN(Register, Value8),
    I5XY0(Register, Register),
    I6XNN(Register, Value8),
    I7XNN(Register, Value8),
    I8XY0(Register, Register),
    I8XY1(Register, Register),
    I8XY2(Register, Register),
    I8XY3(Register, Register),
    I8XY4(Register, Register),
    I8XY5(Register, Register),
    I8XY6(Register, Register),
    I8XY7(Register, Register),
    I8XYE(Register, Register),
    I9XY0(Register, Register),
    IANNN(Address),
    IBNNN(Address),
    ICXNN(Register, Value8),
    IDXYN(Register, Register, Value4),
    IEX9E(Register),
    IEXA1(Register),
    IFX07(Register),
    IFX0A(Register),
    IFX15(Register),
    IFX18(Register),
    IFX1E(Register),
    IFX29(Register),
    IFX33(Register),
    IFX55(RegisterRange),
    IFX65(RegisterRange),
}

impl Instruction {
    fn decode_0(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        
        match ins {
            0x00E0 => Ok(I00E0),
            0x00EE => Ok(I00EE),
            0x0000..=0x0FFFF => Ok(I0NNN(Address::from(ins)))
        }
    }
    
    fn decode_1(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I1NNN(Address::from(ins)))
    }
    
    fn decode_2(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I2NNN(Address::from(ins)))
    }
    
    fn decode_3(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I3XNN(Register::from(ins >> 8), Value8::from(ins)))
    }
    
    fn decode_4(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I4XNN(Register::from(ins >> 8), Value8::from(ins)))
    }
    
    fn decode_5(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        match ins.nibbles() {
            (0x5, x, y, 0x0) => Ok(I5XY0(Register::from(x), Register::from(y))),
            _ => Err(Error::InvalidInstruction),
        }
    }
    
    fn decode_6(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I6XNN(Register::from(ins >> 8), Value8::from(ins)))
    }
    
    fn decode_7(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(I7XNN(Register::from(ins >> 8), Value8::from(ins)))
    }
    
    fn decode_8(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        match ins.nibbles() {
            (8, x, y, 0x0) => Ok(I8XY0(Register::from(x), Register::from(y))),
            (8, x, y, 0x1) => Ok(I8XY1(Register::from(x), Register::from(y))),
            (8, x, y, 0x2) => Ok(I8XY2(Register::from(x), Register::from(y))),
            (8, x, y, 0x3) => Ok(I8XY3(Register::from(x), Register::from(y))),
            (8, x, y, 0x4) => Ok(I8XY4(Register::from(x), Register::from(y))),
            (8, x, y, 0x5) => Ok(I8XY5(Register::from(x), Register::from(y))),
            (8, x, y, 0x6) => Ok(I8XY6(Register::from(x), Register::from(y))),
            (8, x, y, 0x7) => Ok(I8XY7(Register::from(x), Register::from(y))),
            (8, x, y, 0xE) => Ok(I8XYE(Register::from(x), Register::from(y))),
            _ => Err(Error::InvalidInstruction),
        }
    }
    
    fn decode_9(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        match ins.nibbles() {
            (9, x, y, 0) => Ok(I9XY0(Register::from(x), Register::from(y))),
            _ => Err(Error::InvalidInstruction),
        }
    }
    
    fn decode_a(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(IANNN(Address::from(ins)))
    }
    
    fn decode_b(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(IBNNN(Address::from(ins)))
    }
    
    fn decode_c(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(ICXNN(Register::from(ins >> 8), Value8::from(ins)))
    }
    
    fn decode_d(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        Ok(IDXYN(Register::from(ins >> 8), Register::from(ins >> 4), Value4::from(ins)))
    }
    
    fn decode_e(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        match ins.nibbles() {
            (0xE, x, 0x9, 0xE) => Ok(IEX9E(Register::from(x))),
            (0xE, x, 0xA, 0x1) => Ok(IEXA1(Register::from(x))),
            _ => Err(Error::InvalidInstruction),
        }
    }
    
    fn decode_f(ins: u16) -> Result<Self, Error> {
        use Instruction::*;
        match ins.nibbles() {
            (0xF, x, 0x0, 0x7) => Ok(IFX07(Register::from(x))),
            (0xF, x, 0x0, 0xA) => Ok(IFX0A(Register::from(x))),
            (0xF, x, 0x1, 0x5) => Ok(IFX15(Register::from(x))),
            (0xF, x, 0x1, 0x8) => Ok(IFX18(Register::from(x))),
            (0xF, x, 0x1, 0xE) => Ok(IFX1E(Register::from(x))),
            (0xF, x, 0x2, 0x9) => Ok(IFX29(Register::from(x))),
            (0xF, x, 0x3, 0x3) => Ok(IFX33(Register::from(x))),
            (0xF, x, 0x5, 0x5) => Ok(IFX55(RegisterRange::from(x))),
            (0xF, x, 0x6, 0x5) => Ok(IFX65(RegisterRange::from(x))),
            _ => Err(Error::InvalidInstruction),
        }
    }
}

impl TryFrom<[u8; 2]> for Instruction {
    type Error = Error;
    
    fn try_from(instruction: [u8; 2]) -> Result<Self, Error> {
        let ins = u16::from_be_bytes(instruction);
        match ins {
            0x0000..=0x0FFF => Self::decode_0(ins),
            0x1000..=0x1FFF => Self::decode_1(ins),
            0x2000..=0x2FFF => Self::decode_2(ins),
            0x3000..=0x3FFF => Self::decode_3(ins),
            0x4000..=0x4FFF => Self::decode_4(ins),
            0x5000..=0x5FFF => Self::decode_5(ins),
            0x6000..=0x6FFF => Self::decode_6(ins),
            0x7000..=0x7FFF => Self::decode_7(ins),
            0x8000..=0x8FFF => Self::decode_8(ins),
            0x9000..=0x9FFF => Self::decode_9(ins),
            0xA000..=0xAFFF => Self::decode_a(ins),
            0xB000..=0xBFFF => Self::decode_b(ins),
            0xC000..=0xCFFF => Self::decode_c(ins),
            0xd000..=0xDFFF => Self::decode_d(ins),
            0xE000..=0xEFFF => Self::decode_e(ins),
            0xF000..=0xFFFF => Self::decode_f(ins),
        }
    }
}


#[derive(Debug)]
pub struct Core<'memory> {
    mem: &'memory mut [u8],
    reg: &'memory mut [u8],
    stack: &'memory mut [u16],
    i: u16,
    pc: u16,
    sp: u8,
    wait_for_keypress: bool,
}

impl<'memory> Core<'memory> {
    pub fn new(mem: &'memory mut [u8], reg: &'memory mut [u8], stack: &'memory mut [u16]) -> Self {
        assert!(mem.len() >= 2048);
        assert!(reg.len() >= 16);
        assert!(stack.len() >= 16);
        
        Self {
            mem,
            reg,
            stack,
            i: 0,
            pc: 0,
            sp: 0,
            wait_for_keypress: false,
        }
    }
    
    pub fn tick<G, TD, TS>(&mut self, keys: Keys, graphics: &mut G, timer_delay: &mut TD, timer_sound: &mut TS)
    where
        G: Graphics,
        TD: Timer,
        TS: Timer,
    {
        match (self.wait_for_keypress, keys.pressed()) {
            (true, false) => return,
            (true, true) => self.wait_for_keypress = false,
            _ => (),
        }
    }
}

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
    pub fn new(core: Core<'memory>, freq: u32, keypad: K, graphics: G, timer_delay: TD, timer_sound: TS) -> Self{
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
        self.core.tick(keys, &mut self.graphics, &mut self.timer_delay, &mut self.timer_sound);
    }
    
    fn tick_timers(&mut self) {
        self.timer_delay.tick();
        self.timer_sound.tick();
    }
    
}

fn main() {
    let mut mem = [0; 2048];
    let mut reg = [0; 16];
    let mut stack = [0; 16];

    let mut chip8 = Chip8::new(
        Core::new(&mut mem[..], &mut reg[..], &mut stack[..]),
        60,
        NullKeypad,
        NullGraphics,
        DownTimer(0),
        DownTimer(0),
    );
    
    chip8.tick();
    
    println!("{:?}, {:?}", chip8.timer_delay, chip8.timer_sound);
}