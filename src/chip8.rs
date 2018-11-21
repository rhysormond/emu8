use sprites::SPRITE_SHEET;

/// # Chip-8
/// Chip-8 is a virtual machine and corresponding interpreted language.
///
/// ## CPU
///
/// Registers
/// - 16 primary 8-bit registers (V0..VF)
///     - the first 15 (V0..VE) are general purpose registers
///     - the 16th (VF) is the carry flag
/// - a 16-bit memory address register
///
/// Counter
/// - a 16-bit program counter
///
/// Pointer
/// - a 8-bit stack pointer
///
/// Timers
/// - 2 8-bit timers (delay & sound)
///     - they decrement sequentially once per tick
///     - when the sound timer is above 0 it plays a beep
///
/// ## Memory
/// - 64 byte stack
///     - stores return addresses when subroutines are called
/// - 32x64 byte frame buffer
///     - stores the contents of the next frame to be drawn
/// - 4096 bytes of addressable memory
/// - 4096 bytes of ROM (optional)
pub struct Chip8 {
    v_registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    stack: Stack,
    memory: Memory,
    pub frame_buffer: FrameBuffer,
}

pub type FrameBuffer = [[u8; 32]; 64];
pub type Memory = [u8; 4096];
pub type Stack = [u8; 16];

impl Chip8 {
    pub fn new() -> Self {
        // 0x000 - 0x080 is reserved for a sprite sheet
        let mut memory: Memory = [0; 4096];
        memory[0..80].copy_from_slice(&SPRITE_SHEET);

        // 0x200 is where ROMs are loaded into memory
        let program_counter: u16 = 0x200;

        Chip8 {
            v_registers: [0; 16],
            address_register: 0,
            program_counter,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            memory,
            frame_buffer: [[0; 32]; 64],
        }
    }

    /// Handles input from event loop.
    pub fn key_press(&mut self, key: u8) {
        // TODO Actually handle input
    }

    pub fn key_release(&mut self, key: u8) {
        // TODO Actually handle input
    }

    /// Execute one CPU cycle.
    pub fn cycle(&mut self) {
        // TODO Decrement Delay Timer
        // TODO Decrement Sound Timer
        // TODO Increment Program Counter if it needs it
    }
}
