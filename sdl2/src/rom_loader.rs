use std::{error::Error, fs, path::PathBuf};

pub struct RomLoader;

impl RomLoader {
    const ROMS_FOLDER: &'static str = "./roms";

    pub fn load_rom(rom_name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut rom_path = PathBuf::from(RomLoader::ROMS_FOLDER);
        rom_path.push(rom_name);

        Ok(fs::read(rom_path)?)
    }
}
