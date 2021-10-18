use crate::peripherals::{FallingEdges, Graphics, Keypad, Keys, Pos, Sprite};
use log::debug;
use minifb::{Error, Key, Window, WindowOptions};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
    Arc, Mutex,
};

#[derive(Debug)]
struct Buffer {
    buf: Mutex<Vec<u32>>,
    changed: AtomicBool,
}

#[derive(Debug)]
pub struct CurrentKeys {
    prev: Keys,
    current: Keys,
}

#[derive(Debug)]
pub struct MinifbDisplay {
    window: Window,
    buffer: Arc<Buffer>,
    keys: Arc<Mutex<CurrentKeys>>,
}

fn map_keys(keys: &[Key]) -> Keys {
    let mut final_value = 0;

    for key in keys {
        let val = match key {
            Key::Key1 => 0x1,
            Key::Key2 => 0x2,
            Key::Key3 => 0x3,
            Key::Key4 => 0xC,
            Key::Q => 0x4,
            Key::W => 0x5,
            Key::E => 0x6,
            Key::R => 0xD,
            Key::A => 0x7,
            Key::S => 0x8,
            Key::D => 0x9,
            Key::F => 0xE,
            Key::Z => 0xA,
            Key::X => 0x0,
            Key::C => 0xB,
            Key::V => 0xF,
            _ => 0x0,
        };

        debug!("final_value {}", final_value);
        final_value |= 1 << val;
    }

    Keys(final_value)
}

impl MinifbDisplay {
    const SCALE: usize = 10;

    pub fn new(fps_target: u64) -> Result<Self, Error> {
        let width = GraphicsAdapter::WIDTH * Self::SCALE;
        let height = GraphicsAdapter::HEIGHT * Self::SCALE;

        let mut window = Window::new("CHIP-8 Emulator", width, height, WindowOptions::default())?;

        window.limit_update_rate(Some(std::time::Duration::from_micros(
            1_000_000 / fps_target,
        )));

        let buffer = Buffer {
            buf: Mutex::new(vec![0; width * height]),
            changed: AtomicBool::new(false),
        };

        let current_keys = Mutex::new(CurrentKeys {
            prev: Keys(0),
            current: Keys(0),
        });

        Ok(Self {
            window,
            buffer: Arc::new(buffer),
            keys: Arc::new(current_keys),
        })
    }

    pub fn keypad_adater(&self) -> KeypadAdapter {
        KeypadAdapter(self.keys.clone())
    }

    pub fn graphics_adapter(&self) -> GraphicsAdapter {
        GraphicsAdapter(self.buffer.clone())
    }

    pub fn run(&mut self, stop: Receiver<()>) -> Result<(), Error> {
        let (width, height) = self.window.get_size();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            if let Ok(()) = stop.try_recv() {
                return Ok(());
            }

            let pressed_keys =
                if let Some(pressed_keys) = self.window.get_keys_pressed(minifb::KeyRepeat::Yes) {
                    map_keys(&pressed_keys[..])
                } else {
                    Keys(0)
                };

            {
                let keys = &mut self.keys.lock().expect("Locking keys failed");
                let current = keys.current.clone();
                keys.prev.update(&current);
                keys.current = pressed_keys;
            }

            if self.buffer.changed.swap(false, Ordering::Relaxed) {
                let buffer = {
                    self.buffer
                        .buf
                        .lock()
                        .expect("Locking graphics buffer failed")
                        .clone()
                };

                self.window.update_with_buffer(&buffer, width, height)?;
            } else {
                self.window.update();
            }
        }

        Ok(())
    }

    pub fn set_pixel(buffer: &mut [u32], x: usize, y: usize, on: bool) -> bool {
        let x_first = Self::SCALE * x;
        let y_first = Self::SCALE * y;

        let x_range = x_first..(Self::SCALE * x + Self::SCALE);
        let y_range = y_first..(Self::SCALE * y + Self::SCALE);

        let val = if on { 0xFF_FF_FF } else { 0 };

        let collision = on && buffer[x_first + y_first * GraphicsAdapter::WIDTH * Self::SCALE] != 0;

        for x in x_range {
            for y in y_range.clone() {
                buffer[x + y * GraphicsAdapter::WIDTH * Self::SCALE] ^= val;
            }
        }

        collision
    }

    pub fn reset_pixel(buffer: &mut [u32], x: usize, y: usize) {
        let x_range = (Self::SCALE * x)..(Self::SCALE * x + Self::SCALE);
        let y_range = (Self::SCALE * y)..(Self::SCALE * y + Self::SCALE);

        for x in x_range {
            for y in y_range.clone() {
                buffer[x + y * GraphicsAdapter::WIDTH * Self::SCALE] = 0;
            }
        }
    }
}

#[derive(Debug)]
pub struct KeypadAdapter(Arc<Mutex<CurrentKeys>>);

impl Keypad for KeypadAdapter {
    fn pressed_keys(&self) -> Keys {
        let keys = &self.0.lock().expect("Locking keys buffer failed").current;
        keys.clone()
    }

    fn last_released_key(&mut self) -> FallingEdges {
        let keys = &self.0.lock().expect("Locking keys buffer failed");

        keys.prev.falling_edges(&keys.current)
    }
}

#[derive(Debug)]
pub struct GraphicsAdapter(Arc<Buffer>);

impl Graphics for GraphicsAdapter {
    fn clear(&mut self) {
        let mut buffer = self.0.buf.lock().expect("Locking graphics buffer failed");

        for x in 0..Self::WIDTH {
            for y in 0..Self::HEIGHT {
                MinifbDisplay::reset_pixel(&mut buffer, x, y);
            }
        }
    }

    fn toggle_sprite(&mut self, pos: Pos, sprite: Sprite<'_>) -> bool {
        let mut collision = false;
        let mut buffer = self.0.buf.lock().expect("Locking graphics buffer failed");

        for y in 0..sprite.0.len() {
            for x in 0..8 {
                let x_pos = (pos.0 as usize + x) % Self::WIDTH;
                let y_pos = (pos.1 as usize + y) % Self::HEIGHT;
                let sprite_bit = sprite.0[y] >> (7 - x) as u32 & 0x01 == 1;

                if MinifbDisplay::set_pixel(&mut buffer, x_pos, y_pos, sprite_bit) {
                    collision = true;
                }
            }
        }

        collision
    }

    fn refresh(&mut self) {
        self.0.changed.store(true, Ordering::Relaxed);
    }
}
