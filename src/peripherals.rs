#[derive(Debug)]
pub struct Keys(u16);

impl Keys {
    pub fn pressed(&self) -> bool {
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

#[derive(Debug)]
pub struct Pos(pub u8, pub u8);

#[derive(Debug)]
pub struct Sprite<'memory>(pub &'memory [u8]);

pub trait Graphics {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    fn clear(&mut self);
    fn toggle_sprite(&mut self, pos: Pos, sprite: Sprite<'_>) -> bool;
    fn refresh(&mut self);
}

#[derive(Debug)]
pub struct NullGraphics;

impl Graphics for NullGraphics {
    fn clear(&mut self) {}
    fn toggle_sprite(&mut self, _pos: Pos, _sprite: Sprite<'_>) -> bool {
        false
    }
    fn refresh(&mut self) {}
}

pub trait Timer {
    fn tick(&mut self) -> bool;
    fn get(&self) -> u8;
    fn set(&mut self, val: u8);
}

#[derive(Debug)]
pub struct DownTimer(u8);

impl Default for DownTimer {
    fn default() -> Self {
        Self(0)
    }
}

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
