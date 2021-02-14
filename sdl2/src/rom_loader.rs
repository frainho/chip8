use std::{error::Error, fs, path::PathBuf};

pub struct RomLoader;

impl RomLoader {
    pub fn load_rom<P>(rom_path: P) -> Result<Vec<u8>, Box<dyn Error>>
    where
        P: Into<PathBuf>,
    {
        Ok(fs::read(rom_path.into())?)
    }
}
