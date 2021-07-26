use crate::Error;
use std::convert::{TryFrom, TryInto};
use Instruction::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Register(u8);

impl From<u16> for Register {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterRange(u8);

impl From<u16> for RegisterRange {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Address(u16);

impl From<u16> for Address {
    fn from(val: u16) -> Self {
        Self(val & 0x0FFF)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value8(u8);

impl From<u16> for Value8 {
    fn from(val: u16) -> Self {
        Self((val & 0x00FF) as u8)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value4(u8);

impl From<u16> for Value4 {
    fn from(val: u16) -> Self {
        Self((val & 0x000F) as u8)
    }
}

trait Nibble {
    type Output;
    fn nibbles(&self) -> Self::Output;
}

impl Nibble for u16 {
    type Output = (u16, u16, u16, u16);

    fn nibbles(&self) -> Self::Output {
        (
            (*self >> 12) & 0xF,
            (*self >> 8) & 0xF,
            (*self >> 4) & 0xF,
            *self & 0xF,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
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
        match ins {
            0x00E0 => Ok(I00E0),
            0x00EE => Ok(I00EE),
            0x0200..=0x0FFF => Ok(I0NNN(Address::from(ins))),
            _ => Err(Error::InvalidInstruction),
        }
    }

    fn decode_1(ins: u16) -> Result<Self, Error> {
        Ok(I1NNN(Address::from(ins)))
    }

    fn decode_2(ins: u16) -> Result<Self, Error> {
        Ok(I2NNN(Address::from(ins)))
    }

    fn decode_3(ins: u16) -> Result<Self, Error> {
        Ok(I3XNN(Register::from(ins >> 8), Value8::from(ins)))
    }

    fn decode_4(ins: u16) -> Result<Self, Error> {
        Ok(I4XNN(Register::from(ins >> 8), Value8::from(ins)))
    }

    fn decode_5(ins: u16) -> Result<Self, Error> {
        match ins.nibbles() {
            (0x5, x, y, 0x0) => Ok(I5XY0(Register::from(x), Register::from(y))),
            _ => Err(Error::InvalidInstruction),
        }
    }

    fn decode_6(ins: u16) -> Result<Self, Error> {
        Ok(I6XNN(Register::from(ins >> 8), Value8::from(ins)))
    }

    fn decode_7(ins: u16) -> Result<Self, Error> {
        Ok(I7XNN(Register::from(ins >> 8), Value8::from(ins)))
    }

    fn decode_8(ins: u16) -> Result<Self, Error> {
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
        match ins.nibbles() {
            (9, x, y, 0) => Ok(I9XY0(Register::from(x), Register::from(y))),
            _ => Err(Error::InvalidInstruction),
        }
    }

    fn decode_a(ins: u16) -> Result<Self, Error> {
        Ok(IANNN(Address::from(ins)))
    }

    fn decode_b(ins: u16) -> Result<Self, Error> {
        Ok(IBNNN(Address::from(ins)))
    }

    fn decode_c(ins: u16) -> Result<Self, Error> {
        Ok(ICXNN(Register::from(ins >> 8), Value8::from(ins)))
    }

    fn decode_d(ins: u16) -> Result<Self, Error> {
        Ok(IDXYN(
            Register::from(ins >> 8),
            Register::from(ins >> 4),
            Value4::from(ins),
        ))
    }

    fn decode_e(ins: u16) -> Result<Self, Error> {
        match ins.nibbles() {
            (0xE, x, 0x9, 0xE) => Ok(IEX9E(Register::from(x))),
            (0xE, x, 0xA, 0x1) => Ok(IEXA1(Register::from(x))),
            _ => Err(Error::InvalidInstruction),
        }
    }

    fn decode_f(ins: u16) -> Result<Self, Error> {
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

impl TryFrom<&[u8]> for Instruction {
    type Error = Error;

    fn try_from(instruction: &[u8]) -> Result<Self, Error> {
        let ins = u16::from_be_bytes(instruction.try_into()?);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error::*;
    use std::convert::TryFrom;

    macro_rules! itf_ok {
        ( $upper:expr, $lower:expr, $rhs:expr ) => {
            assert_eq!(Instruction::try_from([$upper, $lower].as_ref()), Ok($rhs));
        };
    }

    macro_rules! itf_err {
        ( $upper:expr, $lower:expr, $rhs:expr ) => {
            assert_eq!(Instruction::try_from([$upper, $lower].as_ref()), Err($rhs));
        };
    }

    #[test]
    fn decode_0_ok() {
        itf_ok!(0x00, 0xE0, I00E0);
        itf_ok!(0x00, 0xEE, I00EE);
        itf_ok!(0x02, 0x00, I0NNN(Address(0x200)));
        itf_ok!(0x0F, 0xFF, I0NNN(Address(0xFFF)));
    }

    #[test]
    fn decode_0_err() {
        itf_err!(0x00, 0x00, InvalidInstruction);
        itf_err!(0x01, 0xFF, InvalidInstruction);
    }
}
