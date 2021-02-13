/// Errors enum used both within the chip8 core and exported for use in a frontend
#[derive(Debug)]
pub enum Chip8Error {
    /// Whether it failed when loading the program into memory
    UnableToLoadProgram,
    /// Whether the program contains an opcode that is not valid
    InvalidOpcode(u16),
    /// Error while trying to draw graphics
    GraphicsError(String),
}

impl std::error::Error for Chip8Error {}

impl std::fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chip8Error::UnableToLoadProgram => write!(f, "Unable to load program"),
            Chip8Error::InvalidOpcode(invalid_opcode) => {
                write!(f, "Invalid opcode: {}", invalid_opcode)
            }
            Chip8Error::GraphicsError(message) => {
                write!(f, "Error while drawing graphics: {}", message)
            }
        }
    }
}

impl From<std::io::Error> for Chip8Error {
    fn from(_: std::io::Error) -> Self {
        Chip8Error::UnableToLoadProgram
    }
}
