pub mod minifb;

use std::io::{self, Read};
use std::path::Path;

pub fn load_program<P: AsRef<Path>>(path: P, target: &mut [u8]) -> io::Result<()> {
    let mut rom = std::fs::File::open(path.as_ref())?;
    let _ = rom.read(&mut target[0x200..])?;

    Ok(())
}
