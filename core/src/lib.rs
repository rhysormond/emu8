pub use chip8::Chip8;
pub use constants::CLOCK_SPEED;
pub use keymap::keymap;

mod chip8;
pub mod constants;
mod instruction;
mod keymap;
mod opcode;
pub mod state;
