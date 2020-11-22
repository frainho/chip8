#![allow(dead_code)]
use std::{fs, fs::File, path::PathBuf};

use crate::types::Memory;

const TEMP_FOLDER: &str = "./tmpdir";

#[cfg(test)]
pub struct TestFile {
    file: File,
}

#[cfg(test)]
impl TestFile {
    pub fn create(name: &str, data: &[u8]) -> Result<TestFile, std::io::Error> {
        let mut file_path = PathBuf::from(TEMP_FOLDER);
        file_path.push(name);

        fs::create_dir_all(TEMP_FOLDER)?;

        let file = File::create(&file_path)?;

        fs::write(&file_path, data)?;
        // file.write(data)?;

        Ok(TestFile { file })
    }
}

#[cfg(test)]
impl Drop for TestFile {
    fn drop(&mut self) {
        match fs::remove_dir_all(TEMP_FOLDER) {
            Ok(_) => println!("File clean up ok"),
            Err(_) => eprintln!("Unable to cleanup file"),
        };
    }
}

pub fn set_initial_opcode_to(opcode: u16, memory: &mut Memory) {
    memory[0x200] = ((opcode & 0xFF00) >> 8) as u8;
    memory[0x201] = (opcode & 0x00FF) as u8;
}
