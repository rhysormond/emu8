use opcode::Opcode;
use sprites::SPRITE_SHEET;

/// # Chip-8
///
/// Chip-8 is a virtual machine and corresponding interpreted language.
///
/// ## CPU
///
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

/// The Chip-8 runs at 500Hz which is equal to two million nanoseconds per cycle
pub const CLOCK_SPEED: usize = 2_000_000;
/// The Chip-8 has a 64x32 pixel display
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
/// The FrameBuffer is indexed as [y][x]
pub type FrameBuffer = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

impl Chip8 {
    pub fn new() -> Self {
        // 0x000 - 0x080 is reserved for a sprite sheet
        let mut memory = [0; 4096];
        memory[0..80].copy_from_slice(&SPRITE_SHEET);

        // 0x200 is where ROMs are loaded into memory
        let pc: u16 = 0x200;

        Chip8 {
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

    /// Load a rom from a source file
    pub fn load_rom(&mut self, file: &mut std::io::Read) {
        file.read(&mut self.memory[0x200..]).unwrap();
    }

    /// Returns the FrameBuffer if the display should be redrawn
    pub fn get_frame(&self) -> Option<FrameBuffer> {
        match self.draw_flag {
            true => Some(self.frame_buffer),
            _ => None,
        }
    }

    /// Set the pressed status of key
    pub fn key_press(&mut self, key: u8) {
        self.pressed_keys[key as usize] = 0x1;
        if let Some(register) = self.register_needing_key {
            self.v[register as usize] = key;
            self.register_needing_key = None;
        }
    }

    /// Unset the pressed status of key
    pub fn key_release(&mut self, key: u8) {
        self.pressed_keys[key as usize] = 0x0;
    }

    /// Executes a single CPU cycle
    /// - breaks if awaiting a keypress
    /// - gets and executes the next opcode
    pub fn cycle_cpu(&mut self) {
        if self.register_needing_key == None {
            let op: u16 = self.get_op();
            self.execute_op(op);
        }
    }

    /// Handles delay counter and timers
    /// - decrements the delay counter
    /// - decrements timers when the counter hits 0
    pub fn cycle_timers(&mut self) {
        if self.delay_counter == 0 {
            // There are approximately 8 CPU cycles per delay increment
            self.delay_counter = 8;

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                // TODO make some sound
                self.sound_timer -= 1;
            }
        } else {
            self.delay_counter -= 1;
        }
    }

    /// Gets the opcode currently pointed at by the pc.
    ///
    /// Memory is stored as bytes, but opcodes are 16 bits so we combine two subsequent bytes.
    fn get_op(&self) -> u16 {
        let left = u16::from(self.memory[self.pc as usize]);
        let right = u16::from(self.memory[self.pc as usize + 1]);
        left << 8 | right
    }

    /// Execute a single opcode
    ///
    /// Match the opcode's nibbles against a table and use them to conditionally edit memory.
    ///
    /// # Arguments
    /// * `op` a 16-bit opcode
    fn execute_op(&mut self, op: u16) {
        // TODO refactor this to eliminate some repetition
        // TODO use a logger instead of print statements
        print!("{:04X} ", op);

        // how much to increment pc after executing the op
        let mut pc_bump: u16 = 0x2;
        match op.nibbles() {
            (0x0, 0x0, 0xE, 0x0) => {
                println!("CLS  | clear");
                self.frame_buffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                self.draw_flag = true;
            }
            (0x0, 0x0, 0xE, 0xE) => {
                println!("RET  | PC = STACK.pop()");
                self.pc = self.stack[self.sp as usize];
                self.sp -= 0x1;
            }
            (0x1, ..) => {
                let addr = op.addr();
                println!("JP   | PC = {:04X}", addr);
                self.pc = addr;
                pc_bump = 0x0;
            }
            (0x2, ..) => {
                let addr = op.addr();
                println!("CALL | STACK.push(PC); PC = {:04X}", addr);
                self.sp += 0x1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = addr;
                pc_bump = 0x0;
            }
            (0x3, x, ..) => {
                let kk = op.byte();
                println!("SE   | if V{:X} == {:X} pc += 2", x, kk);
                if self.v[x as usize] == kk {
                    self.pc += 0x2;
                };
            }
            (0x4, x, ..) => {
                let kk = op.byte();
                println!("SNE  | if V{:X} != {:X} pc += 2", x, kk);
                if self.v[x as usize] != kk {
                    self.pc += 0x2;
                };
            }
            (0x5, x, y, 0x0) => {
                println!("SE   | if V{:X} == V{:X} pc += 2", x, y);
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 0x2;
                };
            }
            (0x6, x, ..) => {
                let kk = op.byte();
                println!("LD   | V{:X} = {:X}", x, kk);
                self.v[x as usize] = kk;
            }
            (0x7, x, ..) => {
                let kk = op.byte();
                // Add kk to Vx, allow for overflow but implicitly drop it
                println!("Add  | V{:X} += {:X}", x, kk);
                let (res, _) = self.v[x as usize].overflowing_add(kk);
                self.v[x as usize] = res;
            }
            (0x8, x, y, 0x0) => {
                println!("LD   | V{:X} = V{:X}", x, y);
                self.v[x as usize] = self.v[y as usize];
            }
            (0x8, x, y, 0x1) => {
                println!("OR   | V{:X} |= V{:X}", x, y);
                self.v[x as usize] |= self.v[y as usize];
            }
            (0x8, x, y, 0x2) => {
                println!("AND  | V{:X} &= V{:X}", x, y);
                self.v[x as usize] &= self.v[y as usize];
            }
            (0x8, x, y, 0x3) => {
                println!("XOR  | V{:X} ^= V{:X}", x, y);
                self.v[x as usize] ^= self.v[y as usize];
            }
            (0x8, x, y, 0x4) => {
                println!("ADD  | V{:X} += V{:X}; VF = overflow", x, y);
                let (res, over) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[0xF] = if over { 0x1 } else { 0x0 };
                self.v[x as usize] = res;
            }
            (0x8, x, y, 0x5) => {
                println!("SUB  | V{:X} -= V{:X}; VF = !underflow", x, y);
                let (res, under) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[0xF] = if under { 0x0 } else { 0x1 };
                self.v[x as usize] = res;
            }
            (0x8, x, _, 0x6) => {
                println!("SHR  | V{:X} /= 2; VF = underflow", x);
                self.v[0xF] = self.v[x as usize] & 0x1;
                self.v[x as usize] /= 0x2;
            }
            (0x8, x, y, 0x7) => {
                println!("SUBN | V{:X} = V{:X} - V{:X}; VF = underflow", x, y, x);
                let (res, under) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[0xF] = if under { 0x0 } else { 0x1 };
                self.v[x as usize] = res;
            }
            (0x8, x, _, 0xE) => {
                println!("SHL  | V{:X} *= 2; VF = overflow", x);
                let (res, over) = self.v[x as usize].overflowing_mul(2);
                self.v[0xF] = if over { 0x1 } else { 0x0 };
                self.v[x as usize] = res;
            }
            (0x9, x, y, 0x0) => {
                println!("SNE  | if V{:X} != V{:X} pc +=2", x, y);
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 0x2
                };
            }
            (0xA, ..) => {
                let addr = op.addr();
                println!("LD   | I = {:04X}", addr);
                self.i = addr;
            }
            (0xB, ..) => {
                let addr = op.addr();
                println!("JP   | PC = V0 + {:04X}", addr);
                self.pc = self.v[0x0] as u16 + addr;
                pc_bump = 0x0;
            }
            (0xC, x, ..) => {
                let kk = op.byte();
                println!("RND  | V{:X} = rand_byte + {:X}", x, kk);
                let rand_byte: u8 = rand::random();
                self.v[x as usize] = rand_byte & kk;
            }
            (0xD, x, y, n) => {
                println!("DRW  | draw_sprite(x=V{:X} y=V{:X} size={:X})", x, y, n);
                // XORs a sprite from memory i..n at position x, y on the FrameBuffer with wrapping.
                // Sets VF if any pixels would be erased
                self.draw_flag = true;
                self.v[0xF] = 0x0;

                for byte in 0..n as usize {
                    let y = (self.v[y as usize] as usize + byte) % DISPLAY_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.v[x as usize] as usize + bit) % DISPLAY_WIDTH;
                        let pixel_value = (self.memory[self.i as usize + byte] >> (7 - bit)) & 1;
                        self.v[0xF] |= pixel_value & self.frame_buffer[y as usize][x as usize];
                        self.frame_buffer[y as usize][x as usize] ^= pixel_value;
                    }
                }
            }
            (0xE, x, 0x9, 0xE) => {
                println!("SKP  | if V{:X}.pressed pc += 2", x);
                if self.pressed_keys[self.v[x as usize] as usize] == 0x1 {
                    self.pc += 0x2;
                };
            }
            (0xE, x, 0xA, 0x1) => {
                println!("SKNP | if !V{:X}.pressed pc += 2", x);
                if self.pressed_keys[self.v[x as usize] as usize] == 0x0 {
                    self.pc += 0x2;
                };
            }
            (0xF, x, 0x0, 0x7) => {
                println!("LD   | V{:X} = DT", x);
                self.v[x as usize] = self.delay_timer;
            }
            (0xF, x, 0x0, 0xA) => {
                println!("LD   | await keypress for V{:X}", x);
                self.register_needing_key = Some(x)
            }
            (0xF, x, 0x1, 0x5) => {
                println!("LD   | DT = V{:X}", x);
                self.delay_timer = self.v[x as usize];
            }
            (0xF, x, 0x1, 0x8) => {
                println!("LD   | ST = V{:X}", x);
                self.sound_timer = self.v[x as usize];
            }
            (0xF, x, 0x1, 0xE) => {
                println!("ADD  | I += V{:X}", x);
                self.i += self.v[x as usize] as u16;
            }
            (0xF, x, 0x2, 0x9) => {
                // Set I to the memory address of the sprite for Vx
                // See sprites::SPRITE_SHEET for more details
                println!("LD   | I = V{:X} * 5", x);
                self.i = self.v[x as usize] as u16 * 5;
            }
            (0xF, x, 0x3, 0x3) => {
                // Store BCD repr of Vx in memory starting at address i
                println!("LD   | mem[I..I+3] = bcd(V{:X})", x);
                let bcd = [
                    (self.v[x as usize] / 100 % 10),
                    (self.v[x as usize] / 10 % 10),
                    (self.v[x as usize] % 10),
                ];
                self.memory[self.i as usize..(self.i + 0x3) as usize].copy_from_slice(&bcd);
            }
            (0xF, x, 0x5, 0x5) => {
                // Fill memory starting at address i with V0..Vx+1
                println!("LD   | mem[I..I+{:X}] = V0..V{:X}", x, x);
                self.memory[self.i as usize..(self.i + 1 + x as u16) as usize]
                    .copy_from_slice(&self.v[0x0 as usize..1 + x as usize]);
            }
            (0xF, x, 0x6, 0x5) => {
                // Fill V0..Vx+1 with memory starting at address i
                println!("LD   | V0..V{:X} = mem[I..I+{:X}]", x, x);
                self.v[0x0 as usize..1 + x as usize].copy_from_slice(
                    &self.memory[self.i as usize..(self.i + 1 + x as u16) as usize],
                );
            }
            other => panic!("Opcode {:?} is not implemented", other),
        }
        self.pc += pc_bump;
    }
}

#[cfg(test)]
mod test_chip8 {
    use super::*;

    #[test]
    fn test_chip8_get_op() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200..0x202].copy_from_slice(&[0xAA, 0xBB]);
        assert_eq!(chip8.get_op(), 0xAABB);
    }

    #[test]
    fn test_00e0_cls() {
        let mut chip8 = Chip8::new();
        chip8.frame_buffer[0][0] = 1;
        chip8.execute_op(0x00E0);
        assert_eq!(chip8.frame_buffer[0][0], 0);
    }

    #[test]
    fn test_00ee_ret() {
        let mut chip8 = Chip8::new();
        chip8.sp = 0x1;
        chip8.stack[chip8.sp as usize] = 0xABCD;
        chip8.execute_op(0x00EE);
        assert_eq!(chip8.sp, 0x0);
        // Add 2 to the program as it's bumped after opcode execution
        assert_eq!(chip8.pc, 0xABCD + 0x2);
    }

    #[test]
    fn test_1nnn_jp() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0x1ABC);
        assert_eq!(chip8.pc, 0x0ABC);
    }

    #[test]
    fn test_2nnn_call() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0xABCD;
        chip8.execute_op(0x2123);
        assert_eq!(chip8.sp, 0x1);
        assert_eq!(chip8.stack[chip8.sp as usize], 0xABCD);
        assert_eq!(chip8.pc, 0x0123);
    }

    #[test]
    fn test_3xkk_se_skips() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.execute_op(0x3111);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_se_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0x3111);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_4xkk_sne_skips() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0x4111);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_sne_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.execute_op(0x4111);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_5xy0_se_skips() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x5120);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_5xy0_se_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.execute_op(0x5120);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_6xkk_ld() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0x6122);
        assert_eq!(chip8.v[0x1], 0x22);
    }

    #[test]
    fn test_7xkk_add() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x1;
        chip8.execute_op(0x7122);
        assert_eq!(chip8.v[0x1], 0x23);
    }

    #[test]
    fn test_8xy0_ld() {
        let mut chip8 = Chip8::new();
        chip8.v[0x2] = 0x1;
        chip8.execute_op(0x8120);
        assert_eq!(chip8.v[0x1], 0x1);
    }

    #[test]
    fn test_8xy1_or() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x6;
        chip8.v[0x2] = 0x3;
        chip8.execute_op(0x8121);
        assert_eq!(chip8.v[0x1], 0x7);
    }

    #[test]
    fn test_8xy2_and() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x6;
        chip8.v[0x2] = 0x3;
        chip8.execute_op(0x8122);
        assert_eq!(chip8.v[0x1], 0x2);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x6;
        chip8.v[0x2] = 0x3;
        chip8.execute_op(0x8123);
        assert_eq!(chip8.v[0x1], 0x5);
    }

    #[test]
    fn test_8xy4_add_nocarry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0xEE;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x8124);
        assert_eq!(chip8.v[0x1], 0xFF);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0xFF;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x8124);
        assert_eq!(chip8.v[0x1], 0x10);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_nocarry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x33;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x8125);
        assert_eq!(chip8.v[0x1], 0x22);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_carry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.v[0x2] = 0x12;
        chip8.execute_op(0x8125);
        assert_eq!(chip8.v[0x1], 0xFF);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy6_shr_lsb() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x5;
        chip8.execute_op(0x8106);
        assert_eq!(chip8.v[0x1], 0x2);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy6_shr_nolsb() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x4;
        chip8.execute_op(0x8106);
        assert_eq!(chip8.v[0x1], 0x2);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy7_subn_nocarry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.v[0x2] = 0x33;
        chip8.execute_op(0x8127);
        assert_eq!(chip8.v[0x1], 0x22);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy7_subn_carry() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x12;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x8127);
        assert_eq!(chip8.v[0x1], 0xFF);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn test_8xye_shl_msb() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0xFF;
        chip8.execute_op(0x810E);
        // 0xFF * 2 = 0x01FE
        assert_eq!(chip8.v[0x1], 0xFE);
        assert_eq!(chip8.v[0xF], 0x1);
    }

    #[test]
    fn test_8xye_shl_nomsb() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x4;
        chip8.execute_op(0x810E);
        assert_eq!(chip8.v[0x1], 0x8);
        assert_eq!(chip8.v[0xF], 0x0);
    }

    #[test]
    fn test_9xy0_sne_skips() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.execute_op(0x9120);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_9xy0_sne_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x11;
        chip8.v[0x2] = 0x11;
        chip8.execute_op(0x9120);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_annn_ld() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xAABC);
        assert_eq!(chip8.i, 0xABC);
    }

    #[test]
    fn test_bnnn_jp() {
        let mut chip8 = Chip8::new();
        chip8.v[0x0] = 0x2;
        chip8.execute_op(0xBABC);
        assert_eq!(chip8.pc, 0xABE);
    }

    // Not testing cxkk as it generates a random number

    #[test]
    fn test_dxyn_drw_draws() {
        let mut chip8 = Chip8::new();
        chip8.v[0x0] = 0x1;
        // Draw the 0x0 sprite with a 1x 1y offset
        chip8.execute_op(0xD005);
        let mut expected: FrameBuffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        expected[1][1..5].copy_from_slice(&[1, 1, 1, 1]);
        expected[2][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[3][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[4][1..5].copy_from_slice(&[1, 0, 0, 1]);
        expected[5][1..5].copy_from_slice(&[1, 1, 1, 1]);
        assert!(chip8
            .frame_buffer
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| a[..] == b[..]));
    }

    #[test]
    fn test_dxyn_drw_collides() {
        let mut chip8 = Chip8::new();
        chip8.frame_buffer[0][0] = 1;
        chip8.execute_op(0xD001);
        assert_eq!(chip8.v[0xF], 0x1)
    }

    #[test]
    fn test_dxyn_drw_xors() {
        let mut chip8 = Chip8::new();
        // 0 1 0 1 -> Set
        chip8.frame_buffer[0][2..6].copy_from_slice(&[0, 1, 0, 1]);
        // 1 1 0 0 -> Draw xor
        chip8.execute_op(0xD005);
        assert_eq!(chip8.frame_buffer[0][2..6], [1, 0, 0, 1])
    }

    #[test]
    fn test_ex9e_skp_skips() {
        let mut chip8 = Chip8::new();
        chip8.key_press(0xE);
        chip8.v[0x1] = 0xE;
        chip8.execute_op(0xE19E);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_ex9e_skp_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xE19E);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_exa1_sknp_skips() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xE1A1);
        assert_eq!(chip8.pc, 0x0204);
    }

    #[test]
    fn test_exa1_sknp_doesntskip() {
        let mut chip8 = Chip8::new();
        chip8.key_press(0xE);
        chip8.v[0x1] = 0xE;
        chip8.execute_op(0xE1A1);
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_fx07_ld() {
        let mut chip8 = Chip8::new();
        chip8.delay_timer = 0xF;
        chip8.execute_op(0xF107);
        assert_eq!(chip8.v[0x1], 0xF);
    }

    #[test]
    fn test_fx0a_ld_doesntcyclecycle() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xF10A);
        assert_eq!(chip8.pc, 0x0202);
        chip8.cycle_cpu();
        assert_eq!(chip8.pc, 0x0202);
    }

    #[test]
    fn test_fx0a_ld_awaitskeypress() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xF10A);
        assert_eq!(chip8.register_needing_key, Some(0x1));
    }

    #[test]
    fn test_fx0a_ld_captureskeypress() {
        let mut chip8 = Chip8::new();
        chip8.execute_op(0xF10A);
        chip8.key_press(0xE);
        // insert a cls opcode so we don't panic at reading from empty memory
        chip8.memory[0x202..0x204].copy_from_slice(&[0x00, 0xE0]);
        chip8.cycle_cpu();
        assert_eq!(chip8.v[0x1], 0xE);
    }

    #[test]
    fn test_fx15_ld() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0xF;
        chip8.execute_op(0xf115);
        assert_eq!(chip8.delay_timer, 0xF);
    }

    #[test]
    fn test_fx18_ld() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0xF;
        chip8.execute_op(0xf118);
        assert_eq!(chip8.sound_timer, 0xF);
    }

    #[test]
    fn test_fx1e_add() {
        let mut chip8 = Chip8::new();
        chip8.i = 0x1;
        chip8.v[0x1] = 0x1;
        chip8.execute_op(0xF11E);
        assert_eq!(chip8.i, 0x2);
    }

    #[test]
    fn test_fx29_ld() {
        let mut chip8 = Chip8::new();
        chip8.v[0x1] = 0x2;
        chip8.execute_op(0xF129);
        assert_eq!(chip8.i, 0xA);
    }

    #[test]
    fn test_fx33_ld() {
        let mut chip8 = Chip8::new();
        // 0x7B -> 123
        chip8.v[0x1] = 0x7B;
        chip8.i = 0x200;
        chip8.execute_op(0xF133);
        assert_eq!(chip8.memory[0x200..0x203], [0x1, 0x2, 0x3]);
    }

    #[test]
    fn test_fx_55_ld() {
        let mut chip8 = Chip8::new();
        chip8.i = 0x200;
        chip8.v[0x0..0x5].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        chip8.execute_op(0xF455);
        assert_eq!(chip8.memory[0x200..0x205], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }

    #[test]
    fn test_fx_65_ld() {
        let mut chip8 = Chip8::new();
        chip8.i = 0x200;
        chip8.memory[0x200..0x205].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        chip8.execute_op(0xF465);
        assert_eq!(chip8.v[0x0..0x5], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }
}
