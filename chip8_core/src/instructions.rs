use crate::Error;
use Instruction::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register(pub(crate) u8);

impl From<u8> for Register {
    fn from(val: u8) -> Self {
        Self(val & 0x0F)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address(pub(crate) u16);

impl From<(u8, u8, u8)> for Address {
    fn from(val: (u8, u8, u8)) -> Self {
        Self(((val.0 as u16) << 8) & 0xF00 | ((val.1 as u16) << 4) & 0x0F0 | (val.2 as u16) & 0x00F)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:03X}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value8(pub(crate) u8);

impl From<(u8, u8)> for Value8 {
    fn from(val: (u8, u8)) -> Self {
        Self((val.0 << 4) | (val.1 & 0x0F))
    }
}

impl std::fmt::Display for Value8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Value4(pub(crate) u8);

impl From<u8> for Value4 {
    fn from(val: u8) -> Self {
        Self(val & 0x0F)
    }
}

impl std::fmt::Display for Value4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
    IFX55(Register),
    IFX65(Register),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            I0NNN(nnn) => write!(f, "SYS {}", nnn),
            I00E0 => write!(f, "CLS"),
            I00EE => write!(f, "RET"),
            I1NNN(nnn) => write!(f, "JP {}", nnn),
            I2NNN(nnn) => write!(f, "CALL {}", nnn),
            I3XNN(x, vv) => write!(f, "SE {}, {}", x, vv),
            I4XNN(x, vv) => write!(f, "SNE {}, {}", x, vv),
            I5XY0(x, y) => write!(f, "SE {}, {}", x, y),
            I6XNN(x, vv) => write!(f, "LD {}, {}", x, vv),
            I7XNN(x, vv) => write!(f, "ADD {}, {}", x, vv),
            I8XY0(x, y) => write!(f, "LD {}, {}", x, y),
            I8XY1(x, y) => write!(f, "OR {}, {}", x, y),
            I8XY2(x, y) => write!(f, "AND {}, {}", x, y),
            I8XY3(x, y) => write!(f, "XOR {}, {}", x, y),
            I8XY4(x, y) => write!(f, "ADD {}, {}", x, y),
            I8XY5(x, y) => write!(f, "SUB {}, {}", x, y),
            I8XY6(x, y) => write!(f, "SHR {} {{,{}}}", x, y),
            I8XY7(x, y) => write!(f, "SUBN {}, {}", x, y),
            I8XYE(x, y) => write!(f, "SHL {} {{,{}}}", x, y),
            I9XY0(x, y) => write!(f, "SNE {}, {}", x, y),
            IANNN(nnn) => write!(f, "LD I, {}", nnn),
            IBNNN(nnn) => write!(f, "JP V0, {}", nnn),
            ICXNN(x, vv) => write!(f, "RND {}, {}", x, vv),
            IDXYN(x, y, v) => write!(f, "DRW {}, {}, {}", x, y, v),
            IEX9E(x) => write!(f, "SKP {}", x),
            IEXA1(x) => write!(f, "SKNP {}", x),
            IFX07(x) => write!(f, "LD {}, DT", x),
            IFX0A(x) => write!(f, "LD {}, K", x),
            IFX15(x) => write!(f, "LD DT, {}", x),
            IFX18(x) => write!(f, "LD ST, {}", x),
            IFX1E(x) => write!(f, "ADD I, {}", x),
            IFX29(x) => write!(f, "LD F, {}", x),
            IFX33(x) => write!(f, "LD B, {}", x),
            IFX55(x) => write!(f, "LD [I], {}", x),
            IFX65(x) => write!(f, "LD {}, [I]", x),
        }
    }
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
            Value8(0x55) => Ok(IFX55(x)),
            Value8(0x65) => Ok(IFX65(x)),
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
