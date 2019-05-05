use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, SPRITE_SHEET};

/// A snapshot of the Chip8 internal state
///
/// ## CPU
/// Registers
/// - (v) 16 primary 8-bit registers (V0..VF)
///     - the first 15 (V0..VE) are general purpose registers
///     - the 16th (VF) is the carry flag
/// - (i) a 16-bit memory address register
///
/// Counter
/// - (pc) a 16-bit program counter
///
/// Pointer
/// - (sp) a 8-bit stack pointer
///
/// Timers
/// - 2 8-bit timers (delay & sound)
/// - When the sound timer is decremented it plays a beep
///
/// ## Memory
/// - 32 byte stack
///     - stores return addresses when subroutines are called
///     - conflicting sources cite the size as being anywhere from 32-64 bytes
/// - 4096 bytes of addressable memory
/// - 32x64 byte frame buffer
///     - stores the contents of the next frame to be drawn
///
/// ## Input
/// - 16-bit array to track the pressed status of keys 0..F
/// - Emulation may halt until a key's value is written to Some register
///
/// ## Timing
/// - The CPU should have a clock speed of 500Hz
/// - The timers should be decremented at 60Hz
///     - this is approximated as once every 8 CPU cycles
#[derive(Copy, Clone)]
pub struct State {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u16; 16],
    pub memory: [u8; 4096],
    pub frame_buffer: FrameBuffer,
    pub draw_flag: bool,
    pub pressed_keys: [u8; 16],
    pub register_needing_key: Option<u8>,
    pub delay_counter: u8,
}

impl State {
    pub fn new() -> Self {
        // 0x000 - 0x080 is reserved for a sprite sheet
        let mut memory = [0; 4096];
        memory[0..80].copy_from_slice(&SPRITE_SHEET);

        // 0x200 is where ROMs are loaded into memory
        let pc: u16 = 0x200;

        State {
            v: [0; 16],
            i: 0,
            pc,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            memory,
            frame_buffer: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            draw_flag: false,
            pressed_keys: [0; 16],
            register_needing_key: None,
            delay_counter: 0,
        }
    }
}

/// The FrameBuffer is indexed as [y][x]
pub type FrameBuffer = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
