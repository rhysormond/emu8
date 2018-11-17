/// # Chip-8 CPU
///
/// # Registers
/// - 16 primary 8-bit registers (V0..VF)
///     - the first 15 (V0..VE) are general purpose registers
///     - the 16th (VF) is the carry flag
/// - a 16-bit memory address register
///
/// # Counter
/// - a 16-bit program counter
///
/// # Pointer
/// - a 8-bit stack pointer
///
/// # Timers
/// - 2 8-bit timers (delay & sound)
///     - they decrement sequentially once per tick
///     - when the sound timer is above 0 it plays a beep
pub struct CPU {
    pub registers: [u8; 16],
    pub address_register: u16,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: [0; 16],
            address_register: 0,
            program_counter: 0,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}
