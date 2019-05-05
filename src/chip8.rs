use opcode::Opcode;
use sprites::SPRITE_SHEET;
use std::collections::VecDeque;

/// # Chip-8
///
/// Chip-8 is a virtual machine and corresponding interpreted language.
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
pub struct Chip8 {
    state: State,
    previous_states: VecDeque<State>,
}

/// A snapshot of the Chip8 internal state
#[derive(Copy, Clone)]
pub struct State {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    memory: [u8; 4096],
    frame_buffer: FrameBuffer,
    draw_flag: bool,
    pressed_keys: [u8; 16],
    register_needing_key: Option<u8>,
    delay_counter: u8,
}

impl State {
    fn new() -> Self {
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

/// The Chip-8 runs at 500Hz which is equal to two million nanoseconds per cycle
pub const CLOCK_SPEED: usize = 2_000_000;
/// The Chip-8 has a 64x32 pixel display
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
/// The FrameBuffer is indexed as [y][x]
pub type FrameBuffer = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
/// The maximum number of saved states to store
pub const MAX_SAVED_STATES: usize = 1_000;

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            state: State::new(),
            previous_states: VecDeque::with_capacity(MAX_SAVED_STATES),
        }
    }

    /// Load a rom from a source file
    pub fn load_rom(&mut self, file: &mut std::io::Read) {
        file.read(&mut self.state.memory[0x200..]).unwrap();
    }

    /// Returns the FrameBuffer if the display should be redrawn
    pub fn get_frame(&self) -> Option<FrameBuffer> {
        match self.state.draw_flag {
            true => Some(self.state.frame_buffer),
            _ => None,
        }
    }

    /// Set the pressed status of key
    pub fn key_press(&mut self, key: u8) {
        self.state.pressed_keys[key as usize] = 0x1;
        if let Some(register) = self.state.register_needing_key {
            self.state.v[register as usize] = key;
            self.state.register_needing_key = None;
        }
    }

    /// Unset the pressed status of key
    pub fn key_release(&mut self, key: u8) {
        self.state.pressed_keys[key as usize] = 0x0;
    }

    /// Advances the CPU by a single cycle
    /// - breaks if awaiting a keypress
    /// - gets and executes the next opcode
    pub fn advance_cycle(&mut self) {
        if self.state.register_needing_key == None {
            let op: u16 = self.get_op();
            self.state = Chip8::execute_op(op, &self.state);
        };
        self.save_state();
    }

    /// Reverses the CPU by a single cycle if possible
    /// - if there are saved states pops the last one and restores it
    pub fn reverse_cycle(&mut self) {
        self.load_state();
    }

    /// Puts the current state in saved_states
    /// - if there are already MAX_SAVED_STATES saved then the oldest is dropped
    fn save_state(&mut self) {
        if self.previous_states.len() == MAX_SAVED_STATES {
            self.previous_states.pop_back();
        }
        self.previous_states.push_front(self.state);
    }

    /// Puts the current state in saved_states
    /// - if there are already MAX_SAVED_STATES saved then the oldest is dropped
    fn load_state(&mut self) {
        self.previous_states.pop_front().map(|state| self.state = state);
    }

    /// Handles delay counter and timers
    /// - decrements the delay counter
    /// - decrements timers when the counter hits 0
    pub fn cycle_timers(&mut self) {
        if self.state.delay_counter == 0 {
            // There are approximately 8 CPU cycles per delay increment
            self.state.delay_counter = 8;

            if self.state.delay_timer > 0 {
                self.state.delay_timer -= 1;
            }

            if self.state.sound_timer > 0 {
                // TODO make some sound
                self.state.sound_timer -= 1;
            }
        } else {
            self.state.delay_counter -= 1;
        }
    }

    /// Gets the opcode currently pointed at by the pc.
    ///
    /// Memory is stored as bytes, but opcodes are 16 bits so we combine two subsequent bytes.
    fn get_op(&self) -> u16 {
        let left = u16::from(self.state.memory[self.state.pc as usize]);
        let right = u16::from(self.state.memory[self.state.pc as usize + 1]);
        left << 8 | right
    }

    /// Execute a single opcode
    ///
    /// Matches the opcode's nibbles against a table and use them to conditionally edit memory.
    /// Returns a copy of the state after transforming it's transformed by the opcode execution.
    ///
    /// # Arguments
    /// * `op` a 16-bit opcode
    /// * `state` a reference to the Chip-8's internal state
    fn execute_op(op: u16, state: &State) -> State {
        let mut state: State = State::clone(state);
        // TODO refactor this to eliminate some repetition
        // TODO use a logger instead of print statements
        print!("{:04X} ", op);

        // how much to increment pc after executing the op
        let mut pc_bump: u16 = 0x2;
        match op.nibbles() {
            (0x0, 0x0, 0xE, 0x0) => {
                println!("CLS  | clear");
                state.frame_buffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                state.draw_flag = true;
            }
            (0x0, 0x0, 0xE, 0xE) => {
                println!("RET  | PC = STACK.pop()");
                state.pc = state.stack[state.sp as usize];
                state.sp -= 0x1;
            }
            (0x1, ..) => {
                let addr = op.addr();
                println!("JP   | PC = {:04X}", addr);
                state.pc = addr;
                pc_bump = 0x0;
            }
            (0x2, ..) => {
                let addr = op.addr();
                println!("CALL | STACK.push(PC); PC = {:04X}", addr);
                state.sp += 0x1;
                state.stack[state.sp as usize] = state.pc;
                state.pc = addr;
                pc_bump = 0x0;
            }
            (0x3, x, ..) => {
                let kk = op.byte();
                println!("SE   | if V{:X} == {:X} pc += 2", x, kk);
                if state.v[x as usize] == kk {
                    state.pc += 0x2;
                };
            }
            (0x4, x, ..) => {
                let kk = op.byte();
                println!("SNE  | if V{:X} != {:X} pc += 2", x, kk);
                if state.v[x as usize] != kk {
                    state.pc += 0x2;
                };
            }
            (0x5, x, y, 0x0) => {
                println!("SE   | if V{:X} == V{:X} pc += 2", x, y);
                if state.v[x as usize] == state.v[y as usize] {
                    state.pc += 0x2;
                };
            }
            (0x6, x, ..) => {
                let kk = op.byte();
                println!("LD   | V{:X} = {:X}", x, kk);
                state.v[x as usize] = kk;
            }
            (0x7, x, ..) => {
                let kk = op.byte();
                // Add kk to Vx, allow for overflow but implicitly drop it
                println!("Add  | V{:X} += {:X}", x, kk);
                let (res, _) = state.v[x as usize].overflowing_add(kk);
                state.v[x as usize] = res;
            }
            (0x8, x, y, 0x0) => {
                println!("LD   | V{:X} = V{:X}", x, y);
                state.v[x as usize] = state.v[y as usize];
            }
            (0x8, x, y, 0x1) => {
                println!("OR   | V{:X} |= V{:X}", x, y);
                state.v[x as usize] |= state.v[y as usize];
            }
            (0x8, x, y, 0x2) => {
                println!("AND  | V{:X} &= V{:X}", x, y);
                state.v[x as usize] &= state.v[y as usize];
            }
            (0x8, x, y, 0x3) => {
                println!("XOR  | V{:X} ^= V{:X}", x, y);
                state.v[x as usize] ^= state.v[y as usize];
            }
            (0x8, x, y, 0x4) => {
                println!("ADD  | V{:X} += V{:X}; VF = overflow", x, y);
                let (res, over) = state.v[x as usize].overflowing_add(state.v[y as usize]);
                state.v[0xF] = if over { 0x1 } else { 0x0 };
                state.v[x as usize] = res;
            }
            (0x8, x, y, 0x5) => {
                println!("SUB  | V{:X} -= V{:X}; VF = !underflow", x, y);
                let (res, under) = state.v[x as usize].overflowing_sub(state.v[y as usize]);
                state.v[0xF] = if under { 0x0 } else { 0x1 };
                state.v[x as usize] = res;
            }
            (0x8, x, _, 0x6) => {
                println!("SHR  | V{:X} /= 2; VF = underflow", x);
                state.v[0xF] = state.v[x as usize] & 0x1;
                state.v[x as usize] /= 0x2;
            }
            (0x8, x, y, 0x7) => {
                println!("SUBN | V{:X} = V{:X} - V{:X}; VF = underflow", x, y, x);
                let (res, under) = state.v[y as usize].overflowing_sub(state.v[x as usize]);
                state.v[0xF] = if under { 0x0 } else { 0x1 };
                state.v[x as usize] = res;
            }
            (0x8, x, _, 0xE) => {
                println!("SHL  | V{:X} *= 2; VF = overflow", x);
                let (res, over) = state.v[x as usize].overflowing_mul(2);
                state.v[0xF] = if over { 0x1 } else { 0x0 };
                state.v[x as usize] = res;
            }
            (0x9, x, y, 0x0) => {
                println!("SNE  | if V{:X} != V{:X} pc +=2", x, y);
                if state.v[x as usize] != state.v[y as usize] {
                    state.pc += 0x2
                };
            }
            (0xA, ..) => {
                let addr = op.addr();
                println!("LD   | I = {:04X}", addr);
                state.i = addr;
            }
            (0xB, ..) => {
                let addr = op.addr();
                println!("JP   | PC = V0 + {:04X}", addr);
                state.pc = state.v[0x0] as u16 + addr;
                pc_bump = 0x0;
            }
            (0xC, x, ..) => {
                let kk = op.byte();
                println!("RND  | V{:X} = rand_byte + {:X}", x, kk);
                let rand_byte: u8 = rand::random();
                state.v[x as usize] = rand_byte & kk;
            }
            (0xD, x, y, n) => {
                println!("DRW  | draw_sprite(x=V{:X} y=V{:X} size={:X})", x, y, n);
                // XORs a sprite from memory i..n at position x, y on the FrameBuffer with wrapping.
                // Sets VF if any pixels would be erased
                state.draw_flag = true;
                state.v[0xF] = 0x0;

                for byte in 0..n as usize {
                    let y = (state.v[y as usize] as usize + byte) % DISPLAY_HEIGHT;
                    for bit in 0..8 {
                        let x = (state.v[x as usize] as usize + bit) % DISPLAY_WIDTH;
                        let pixel_value = (state.memory[state.i as usize + byte] >> (7 - bit)) & 1;
                        state.v[0xF] |= pixel_value & state.frame_buffer[y as usize][x as usize];
                        state.frame_buffer[y as usize][x as usize] ^= pixel_value;
                    }
                }
            }
            (0xE, x, 0x9, 0xE) => {
                println!("SKP  | if V{:X}.pressed pc += 2", x);
                if state.pressed_keys[state.v[x as usize] as usize] == 0x1 {
                    state.pc += 0x2;
                };
            }
            (0xE, x, 0xA, 0x1) => {
                println!("SKNP | if !V{:X}.pressed pc += 2", x);
                if state.pressed_keys[state.v[x as usize] as usize] == 0x0 {
                    state.pc += 0x2;
                };
            }
            (0xF, x, 0x0, 0x7) => {
                println!("LD   | V{:X} = DT", x);
                state.v[x as usize] = state.delay_timer;
            }
            (0xF, x, 0x0, 0xA) => {
                println!("LD   | await keypress for V{:X}", x);
                state.register_needing_key = Some(x)
            }
            (0xF, x, 0x1, 0x5) => {
                println!("LD   | DT = V{:X}", x);
                state.delay_timer = state.v[x as usize];
            }
            (0xF, x, 0x1, 0x8) => {
                println!("LD   | ST = V{:X}", x);
                state.sound_timer = state.v[x as usize];
            }
            (0xF, x, 0x1, 0xE) => {
                println!("ADD  | I += V{:X}", x);
                state.i += state.v[x as usize] as u16;
            }
            (0xF, x, 0x2, 0x9) => {
                // Set I to the memory address of the sprite for Vx
                // See sprites::SPRITE_SHEET for more details
                println!("LD   | I = V{:X} * 5", x);
                state.i = state.v[x as usize] as u16 * 5;
            }
            (0xF, x, 0x3, 0x3) => {
                // Store BCD repr of Vx in memory starting at address i
                println!("LD   | mem[I..I+3] = bcd(V{:X})", x);
                let bcd = [
                    (state.v[x as usize] / 100 % 10),
                    (state.v[x as usize] / 10 % 10),
                    (state.v[x as usize] % 10),
                ];
                state.memory[state.i as usize..(state.i + 0x3) as usize].copy_from_slice(&bcd);
            }
            (0xF, x, 0x5, 0x5) => {
                // Fill memory starting at address i with V0..Vx+1
                println!("LD   | mem[I..I+{:X}] = V0..V{:X}", x, x);
                state.memory[state.i as usize..(state.i + 1 + x as u16) as usize]
                    .copy_from_slice(&state.v[0x0 as usize..1 + x as usize]);
            }
            (0xF, x, 0x6, 0x5) => {
                // Fill V0..Vx+1 with memory starting at address i
                println!("LD   | V0..V{:X} = mem[I..I+{:X}]", x, x);
                state.v[0x0 as usize..1 + x as usize].copy_from_slice(
                    &state.memory[state.i as usize..(state.i + 1 + x as u16) as usize],
                );
            }
            other => panic!("Opcode {:?} is not implemented", other),
        }
        state.pc += pc_bump;
        state
    }
}

#[cfg(test)]
mod test_chip8 {
    use super::*;

    #[test]
    fn test_chip8_get_op() {
        let mut chip8 = Chip8::new();
        chip8.state.memory[0x200..0x202].copy_from_slice(&[0xAA, 0xBB]);
        assert_eq!(chip8.get_op(), 0xAABB);
    }

    #[test]
    fn test_cycles_while_no_register_needs_key() {
        let mut chip8 = Chip8::new();
        let starting_pc = chip8.state.pc;
        // insert a cls opcode so we don't panic at reading from empty memory
        chip8.state.memory[0x200..0x202].copy_from_slice(&[0x00, 0xE0]);
        chip8.advance_cycle();
        assert_eq!(chip8.state.pc, starting_pc + 0x2);
    }

    #[test]
    fn test_captures_key_presses() {
        let mut chip8 = Chip8::new();
        chip8.state.register_needing_key = Some(0x1);
        chip8.key_press(0xE);
        assert_eq!(chip8.state.register_needing_key, None);
        assert_eq!(chip8.state.v[0x1], 0xE);
    }

    #[test]
    fn test_doesnt_cycle_while_register_needs_key() {
        let mut chip8 = Chip8::new();
        let starting_pc = chip8.state.pc;
        chip8.state.register_needing_key = Some(0x1);
        chip8.advance_cycle();
        assert_eq!(chip8.state.pc, starting_pc);
    }

    #[test]
    fn test_chip8_saves_state() {
        let mut chip8 = Chip8::new();
        let saved_state = chip8.state;
        chip8.save_state();
        assert_eq!(chip8.previous_states.len(), 1);
    }

    #[test]
    fn test_chip8_drops_old_saved_states() {
        let mut chip8 = Chip8::new();
        for _ in 0..MAX_SAVED_STATES {
            chip8.save_state();
        };
        assert_eq!(MAX_SAVED_STATES, chip8.previous_states.len());
        chip8.save_state();
        assert_eq!(MAX_SAVED_STATES, chip8.previous_states.len());
    }

    #[test]
    fn test_chip8_loads_states() {
        let mut chip8 = Chip8::new();
        let saved_state = chip8.state;
        chip8.previous_states.push_front(saved_state);
        chip8.state.delay_counter += 1;
        assert_eq!(chip8.state.delay_counter, 1);
        chip8.load_state();
        assert_eq!(chip8.state.delay_counter, 0);
    }
}

// TODO this should be in another file
#[cfg(test)]
mod test_opcodes {
    use super::*;

    #[test]
    fn test_00e0_cls() {
        let mut state = State::new();
        state.frame_buffer[0][0] = 1;
        let state = Chip8::execute_op(0x00E0, &state);
        assert_eq!(state.frame_buffer[0][0], 0);
    }

    #[test]
    fn test_00ee_ret() {
        let mut state = State::new();
        state.sp = 0x1;
        state.stack[state.sp as usize] = 0xABCD;
        let state = Chip8::execute_op(0x00EE, &state);
        assert_eq!(state.sp, 0x0);
        // Add 2 to the program as it's bumped after opcode execution
        assert_eq!(state.pc, 0xABCD + 0x2);
    }

    #[test]
    fn test_1nnn_jp() {
        let mut state = State::new();
        let state = Chip8::execute_op(0x1ABC, &state);
        assert_eq!(state.pc, 0x0ABC);
    }

    #[test]
    fn test_2nnn_call() {
        let mut state = State::new();
        state.pc = 0xABCD;
        let state = Chip8::execute_op(0x2123, &state);
        assert_eq!(state.sp, 0x1);
        assert_eq!(state.stack[state.sp as usize], 0xABCD);
        assert_eq!(state.pc, 0x0123);
    }

    #[test]
    fn test_3xkk_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = Chip8::execute_op(0x3111, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_se_doesntskip() {
        let mut state = State::new();
        let state = Chip8::execute_op(0x3111, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_4xkk_sne_skips() {
        let mut state = State::new();
        let state = Chip8::execute_op(0x4111, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = Chip8::execute_op(0x4111, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_5xy0_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x5120, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_5xy0_se_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = Chip8::execute_op(0x5120, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_6xkk_ld() {
        let mut state = State::new();
        let state = Chip8::execute_op(0x6122, &state);
        assert_eq!(state.v[0x1], 0x22);
    }

    #[test]
    fn test_7xkk_add() {
        let mut state = State::new();
        state.v[0x1] = 0x1;
        let state = Chip8::execute_op(0x7122, &state);
        assert_eq!(state.v[0x1], 0x23);
    }

    #[test]
    fn test_8xy0_ld() {
        let mut state = State::new();
        state.v[0x2] = 0x1;
        let state = Chip8::execute_op(0x8120, &state);
        assert_eq!(state.v[0x1], 0x1);
    }

    #[test]
    fn test_8xy1_or() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = Chip8::execute_op(0x8121, &state);
        assert_eq!(state.v[0x1], 0x7);
    }

    #[test]
    fn test_8xy2_and() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = Chip8::execute_op(0x8122, &state);
        assert_eq!(state.v[0x1], 0x2);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = Chip8::execute_op(0x8123, &state);
        assert_eq!(state.v[0x1], 0x5);
    }

    #[test]
    fn test_8xy4_add_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0xEE;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x8124, &state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x8124, &state);
        assert_eq!(state.v[0x1], 0x10);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x33;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x8125, &state);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x12;
        let state = Chip8::execute_op(0x8125, &state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy6_shr_lsb() {
        let mut state = State::new();
        state.v[0x1] = 0x5;
        let state = Chip8::execute_op(0x8106, &state);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy6_shr_nolsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = Chip8::execute_op(0x8106, &state);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy7_subn_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x33;
        let state = Chip8::execute_op(0x8127, &state);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy7_subn_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x12;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x8127, &state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xye_shl_msb() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        let state = Chip8::execute_op(0x810E, &state);
        // 0xFF * 2 = 0x01FE
        assert_eq!(state.v[0x1], 0xFE);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xye_shl_nomsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = Chip8::execute_op(0x810E, &state);
        assert_eq!(state.v[0x1], 0x8);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_9xy0_sne_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = Chip8::execute_op(0x9120, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_9xy0_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = Chip8::execute_op(0x9120, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_annn_ld() {
        let mut state = State::new();
        let state = Chip8::execute_op(0xAABC, &state);
        assert_eq!(state.i, 0xABC);
    }

    #[test]
    fn test_bnnn_jp() {
        let mut state = State::new();
        state.v[0x0] = 0x2;
        let state = Chip8::execute_op(0xBABC, &state);
        assert_eq!(state.pc, 0xABE);
    }

    // Not testing cxkk as it generates a random number

    #[test]
    fn test_dxyn_drw_draws() {
        let mut state = State::new();
        state.v[0x0] = 0x1;
        // Draw the 0x0 sprite with a 1x 1y offset
        let state = Chip8::execute_op(0xD005, &state);
        let mut expected: FrameBuffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        expected[1][1..5].copy_from_slice(&[1, 1, 1, 1]);
        expected[2][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[3][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[4][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[5][1..5].copy_from_slice(&[1, 1, 1, 1]);
        assert!(state
            .frame_buffer
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| a[..] == b[..]));
    }

    #[test]
    fn test_dxyn_drw_collides() {
        let mut state = State::new();
        state.frame_buffer[0][0] = 1;
        let state = Chip8::execute_op(0xD001, &state);
        assert_eq!(state.v[0xF], 0x1)
    }

    #[test]
    fn test_dxyn_drw_xors() {
        let mut state = State::new();
        // 0 1 0 1 -> Set
        state.frame_buffer[0][2..6].copy_from_slice(&[0, 1, 0, 1]);
        // 1 1 0 0 -> Draw xor
        let state = Chip8::execute_op(0xD005, &state);
        assert_eq!(state.frame_buffer[0][2..6], [1, 0, 0, 1])
    }

    #[test]
    fn test_ex9e_skp_skips() {
        let mut state = State::new();
        state.pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = Chip8::execute_op(0xE19E, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_ex9e_skp_doesntskip() {
        let mut state = State::new();
        let state = Chip8::execute_op(0xE19E, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_exa1_sknp_skips() {
        let mut state = State::new();
        let state = Chip8::execute_op(0xE1A1, &state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_exa1_sknp_doesntskip() {
        let mut state = State::new();
        state.pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = Chip8::execute_op(0xE1A1, &state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_fx07_ld() {
        let mut state = State::new();
        state.delay_timer = 0xF;
        let state = Chip8::execute_op(0xF107, &state);
        assert_eq!(state.v[0x1], 0xF);
    }

    #[test]
    fn test_fx0a_ld_setsregisterneedingkey() {
        let mut state = State::new();
        let state = Chip8::execute_op(0xF10A, &state);
        assert_eq!(state.register_needing_key, Some(0x1));
    }

    #[test]
    fn test_fx15_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = Chip8::execute_op(0xf115, &state);
        assert_eq!(state.delay_timer, 0xF);
    }

    #[test]
    fn test_fx18_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = Chip8::execute_op(0xf118, &state);
        assert_eq!(state.sound_timer, 0xF);
    }

    #[test]
    fn test_fx1e_add() {
        let mut state = State::new();
        state.i = 0x1;
        state.v[0x1] = 0x1;
        let state = Chip8::execute_op(0xF11E, &state);
        assert_eq!(state.i, 0x2);
    }

    #[test]
    fn test_fx29_ld() {
        let mut state = State::new();
        state.v[0x1] = 0x2;
        let state = Chip8::execute_op(0xF129, &state);
        assert_eq!(state.i, 0xA);
    }

    #[test]
    fn test_fx33_ld() {
        let mut state = State::new();
        // 0x7B -> 123
        state.v[0x1] = 0x7B;
        state.i = 0x200;
        let state = Chip8::execute_op(0xF133, &state);
        assert_eq!(state.memory[0x200..0x203], [0x1, 0x2, 0x3]);
    }

    #[test]
    fn test_fx_55_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.v[0x0..0x5].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = Chip8::execute_op(0xF455, &state);
        assert_eq!(state.memory[0x200..0x205], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }

    #[test]
    fn test_fx_65_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.memory[0x200..0x205].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = Chip8::execute_op(0xF465, &state);
        assert_eq!(state.v[0x0..0x5], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }
}
