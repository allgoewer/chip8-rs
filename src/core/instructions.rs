use crate::Error;
use std::convert::{TryFrom, TryInto};
use Instruction::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Register(pub(crate) u8);

impl From<u8> for Register {
    fn from(val: u8) -> Self {
        Self(val & 0x0F)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterRange(pub(crate) u8);

impl From<u8> for RegisterRange {
    fn from(val: u8) -> Self {
        Self(val & 0x0F)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Address(pub(crate) u16);

impl From<(u8, u8, u8)> for Address {
    fn from(val: (u8, u8, u8)) -> Self {
        Self(
            ((val.0 as u16) << 8) & 0xF00 |
            ((val.1 as u16) << 4) & 0x0F0  |
            (val.2 as u16) & 0x00F
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value8(pub(crate) u8);

impl From<(u8, u8)> for Value8 {
    fn from(val: (u8, u8)) -> Self {
        Self((val.0 << 4) | (val.1 & 0x0F))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Value4(pub(crate) u8);

impl From<u8> for Value4 {
    fn from(val: u8) -> Self {
        Self(val & 0x0F)
    }
}

fn nibbles(val: u16) -> (u8, u8, u8, u8) {
    (
        (val >> 12) as u8,
        (val >> 8) as u8,
        (val >> 4) as u8,
        val as u8,
    )
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
    fn decode_0(nnn: Address) -> Result<Self, ()> {
        match nnn {
            Address(0x00E0) => Ok(I00E0),
            Address(0x00EE) => Ok(I00EE),
            Address(0x0200..=0x0FFF) => Ok(I0NNN(nnn)),
            _ => Err(()),
        }
    }

    fn decode_5(x: Register, y: Register, v: Value4) -> Result<Self, ()> {
        match v { 
            Value4(0) => Ok(I5XY0(x, y)),
            _ => Err(()),
        }
    }

    fn decode_8(x: Register, y: Register, v: Value4) -> Result<Self, ()> {
        match v {
            Value4(0x0) => Ok(I8XY0(x, y)),
            Value4(0x1) => Ok(I8XY1(x, y)),
            Value4(0x2) => Ok(I8XY2(x, y)),
            Value4(0x3) => Ok(I8XY3(x, y)),
            Value4(0x4) => Ok(I8XY4(x, y)),
            Value4(0x5) => Ok(I8XY5(x, y)),
            Value4(0x6) => Ok(I8XY6(x, y)),
            Value4(0x7) => Ok(I8XY7(x, y)),
            Value4(0xE) => Ok(I8XYE(x, y)),
            _ => Err(()),
        }
    }

    fn decode_9(x: Register, y: Register, n: Value4) -> Result<Self, ()> {
        match n {
            Value4(0) => Ok(I9XY0(x, y)),
            _ => Err(()),
        }
    }

    fn decode_e(x: Register, vv: Value8) -> Result<Self, ()> {
        match vv {
            Value8(0x9E) => Ok(IEX9E(x)),
            Value8(0xA1) => Ok(IEXA1(x)),
            _ => Err(()),
        }
    }

    fn decode_f(x: Register, vv: Value8) -> Result<Self, ()> {
        match vv {
            Value8(0x07) => Ok(IFX07(x)),
            Value8(0x0A) => Ok(IFX0A(x)),
            Value8(0x15) => Ok(IFX15(x)),
            Value8(0x18) => Ok(IFX18(x)),
            Value8(0x1E) => Ok(IFX1E(x)),
            Value8(0x29) => Ok(IFX29(x)),
            Value8(0x33) => Ok(IFX33(x)),
            Value8(0x55) => Ok(IFX55(x.0.into())),
            Value8(0x65) => Ok(IFX65(x.0.into())),
            _ => Err(()),
        }
    }
}

impl TryFrom<&[u8]> for Instruction {
    type Error = Error;

    fn try_from(instruction: &[u8]) -> Result<Self, Error> {
        let ins = u16::from_be_bytes(instruction[0..2].try_into()?);
        let decoded = match nibbles(ins) {
            (0x0, a, b, c) => Self::decode_0((a, b, c).into()),
            (0x1, a, b, c) => Ok(I1NNN((a, b, c).into())),
            (0x2, a, b, c) => Ok(I2NNN((a, b, c).into())),
            (0x3, a, b, c) => Ok(I3XNN(a.into(), (b, c).into())),
            (0x4, a, b, c) => Ok(I4XNN(a.into(), (b, c).into())),
            (0x5, a, b, c) => Self::decode_5(a.into(), b.into(), c.into()),
            (0x6, a, b, c) => Ok(I6XNN(a.into(), (b, c).into())),
            (0x7, a, b, c) => Ok(I7XNN(a.into(), (b, c).into())),
            (0x8, a, b, c) => Self::decode_8(a.into(), b.into(), c.into()),
            (0x9, a, b, c) => Self::decode_9(a.into(), b.into(), c.into()),
            (0xA, a, b, c) => Ok(IANNN((a, b, c).into())),
            (0xB, a, b, c) => Ok(IBNNN((a, b, c).into())),
            (0xC, a, b, c) => Ok(ICXNN(a.into(), (b, c).into())),
            (0xD, a, b, c) => Ok(IDXYN(a.into(), b.into(), c.into())),
            (0xE, a, b, c) => Self::decode_e(a.into(), (b, c).into()),
            (0xF, a, b, c) => Self::decode_f(a.into(), (b, c).into()),
            _ => Err(()),
        };

        decoded.map_err(|_| Error::InvalidInstruction(ins))
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
    fn address() {
        assert_eq!(Address::from((0x0A, 0x0B, 0x0C)), Address(0xABC));
        assert_eq!(Address::from((0x1A, 0x2B, 0x4C)), Address(0xABC));
    }

    #[test]
    fn register() {
        assert_eq!(Register::from(0x0A), Register(0x0A));
        assert_eq!(Register::from(0x1A), Register(0x0A));
    }

    #[test]
    fn register_range() {
        assert_eq!(RegisterRange::from(0x0A), RegisterRange(0x0A));
        assert_eq!(RegisterRange::from(0x1A), RegisterRange(0x0A));
    }

    #[test]
    fn value8() {
        assert_eq!(Value8::from((0x0A, 0x0B)), Value8(0xAB));
        assert_eq!(Value8::from((0x1A, 0x2B)), Value8(0xAB));
    }

    #[test]
    fn value4() {
        assert_eq!(Value4::from(0x0A), Value4(0x0A));
        assert_eq!(Value4::from(0x1A), Value4(0x0A));
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
        itf_err!(0x00, 0x00, InvalidInstruction(0x0000));
        itf_err!(0x01, 0xFF, InvalidInstruction(0x01FF));
    }
}
