#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FallingEdges(pub u16);

impl FallingEdges {
    /// Pop the index of the next edge.
    ///
    /// Edges are always popped in ascending order (0 -> 16)
    pub fn pop_next_idx(&mut self) -> Option<u8> {
        if self.0 == 0 {
            return None; // Short circuit immediately if no edge is set
        }

        for i in 0..u16::BITS {
            let bit = 1 << i;
            if self.0 & bit != 0 {
                self.0 ^= bit;
                return Some(i as u8);
            }
        }

        None
    }

    /// Add edges to self
    pub fn push_edges(&mut self, edges: &FallingEdges) {
        self.0 |= edges.0;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keys(pub u16);

impl Keys {
    pub fn pressed(&self, idx: u8) -> bool {
        let bit = 1 << idx;
        self.0 & bit != 0
    }

    pub fn falling_edges(&self, after: &Self) -> FallingEdges {
        FallingEdges(self.0 & !after.0)
    }

    pub fn update(&mut self, after: &Self) -> Option<FallingEdges> {
        let edges = self.falling_edges(after);
        self.0 = after.0;

        if edges.0 == 0 {
            None
        } else {
            Some(edges)
        }
    }
}

pub trait Keypad {
    fn pressed_keys(&self) -> Keys;
    fn last_released_key(&mut self) -> FallingEdges;
}

#[derive(Debug)]
pub struct NullKeypad;

impl Keypad for NullKeypad {
    fn pressed_keys(&self) -> Keys {
        Keys(0)
    }

    fn last_released_key(&mut self) -> FallingEdges {
        FallingEdges(0)
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
pub struct DownTimer<'name> {
    val: u8,
    name: &'name str,
}

impl<'name> DownTimer<'name> {
    pub fn new(name: &'name str) -> Self {
        Self { val: 0, name }
    }
}

impl Timer for DownTimer<'_> {
    fn tick(&mut self) -> bool {
        let (new_val, overflow) = self.val.overflowing_sub(1);
        self.val = new_val;

        #[cfg(feature = "std")]
        if log::log_enabled!(log::Level::Debug) && overflow {
            log::debug!("{} timer overflowed", self.name);
        }

        overflow
    }

    fn get(&self) -> u8 {
        self.val
    }

    fn set(&mut self, val: u8) {
        self.val = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falling_edges() {
        assert_eq!(Keys(0x01).falling_edges(&Keys(0x00)), FallingEdges(0x01));
        assert_eq!(Keys(0x10).falling_edges(&Keys(0x00)), FallingEdges(0x10));
        assert_eq!(Keys(0x11).falling_edges(&Keys(0x00)), FallingEdges(0x11));
        assert_eq!(Keys(0x11).falling_edges(&Keys(0x01)), FallingEdges(0x10));
        assert_eq!(Keys(0x11).falling_edges(&Keys(0x10)), FallingEdges(0x01));
        assert_eq!(Keys(0x11).falling_edges(&Keys(0x11)), FallingEdges(0x00));
    }

    #[test]
    fn keys_update() {
        let mut keys = Keys(0x00);
        assert_eq!(keys, Keys(0x00));

        assert_eq!(keys.update(&Keys(0x01)), None);
        assert_eq!(keys, Keys(0x01));

        assert_eq!(keys.update(&Keys(0x11)), None);
        assert_eq!(keys, Keys(0x11));

        assert_eq!(keys.update(&Keys(0x01)), Some(FallingEdges(0x10)));
        assert_eq!(keys, Keys(0x01));

        assert_eq!(keys.update(&Keys(0x00)), Some(FallingEdges(0x01)));
        assert_eq!(keys, Keys(0x00));

        assert_eq!(keys.update(&Keys(0x11)), None);
        assert_eq!(keys, Keys(0x11));

        assert_eq!(keys.update(&Keys(0x00)), Some(FallingEdges(0x11)));
        assert_eq!(keys, Keys(0x00));
    }

    #[test]
    fn push_pop_edges() {
        let mut edges = FallingEdges(0);

        assert_eq!(edges.pop_next_idx(), None);

        edges.push_edges(&FallingEdges(0x10));
        assert_eq!(edges, FallingEdges(0x10));
        assert_eq!(edges.pop_next_idx(), Some(4));

        edges.push_edges(&FallingEdges(0x10));
        edges.push_edges(&FallingEdges(0x01));

        assert_eq!(edges, FallingEdges(0x11));
        assert_eq!(edges.pop_next_idx(), Some(0));
        assert_eq!(edges.pop_next_idx(), Some(4));
        assert_eq!(edges.pop_next_idx(), None);
    }
}
