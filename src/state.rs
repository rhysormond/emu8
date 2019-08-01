use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, SPRITE_SHEET};

/// # State
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
/// - (delay_timer & sound_timer) 8-bit timers that decrement at 60Hz
///     - each time the sound timer is decremented a beep sound is generated
/// - (delay_counter) how many more CPU cycles should pass before the timers are decremented
///     - the CPU has a clock speed of 500Hz so the timers are decremented once every 8 CPU cycles
///
/// ## Memory
/// - (stack) a 32 byte stack
///     - stores return addresses when subroutines are called
///     - different sources cite the as being anywhere from 32-64 bytes
/// - (memory) 4096 bytes of addressable memory
/// - (frame_buffer) 32x64 bytes of vram
///
/// ## Input
/// - (register_needing_key) whether we're awaiting a keypress so it can be stored in the register
///
/// ## Other
/// - (draw_flag) tracks frame buffer updates since the last draw to prevent unnecessary redraws
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
            register_needing_key: None,
            delay_counter: 0,
        }
    }
}

/// # Frame Buffer
/// Represents the contents of the Chip-8 vram indexed as [y][x].
///
/// Used for:
/// - storing a frame to be drawn to the display
/// - comparisons against the currently drawn sprites for detecting collisions
pub type FrameBuffer = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
