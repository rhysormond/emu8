use opcode::Opcode;
use sprites::SPRITE_SHEET;

/// # Chip-8
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
///     - they decrement sequentially once per tick
///     - when the sound timer is above 0 it plays a beep
///
/// ## Memory
/// - 32 byte stack
///     - stores return addresses when subroutines are called
///     - conflicting sources cite the size as being anywhere from 32-64 bytes
/// - 4096 bytes of addressable memory
/// - 32x64 byte frame buffer
///     - stores the contents of the next frame to be drawn
///
/// ## Rendering
/// - New frames aren't rendered every cycle
///
/// ## Input
/// - 16-bit array to track the pressed status of keys 0..F
/// - Emulation may halt until a key's value is written to Some register
///
pub struct Chip8 {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    memory: [u8; 4096],
    pub frame_buffer: FrameBuffer,
    pub should_draw: bool,
    pressed_keys: [u8; 16],
    register_needing_key: Option<u8>,
}

pub type FrameBuffer = [[u8; 32]; 64];

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
            frame_buffer: [[0; 32]; 64],
            pressed_keys: [0; 16],
            register_needing_key: None,
            should_draw: false,
        }
    }

    /// Load a rom from a source file
    pub fn load_rom(&mut self, file: &mut std::io::Read) {
        file.read(&mut self.memory[0x200..]);
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
    pub fn cycle(&mut self) {
        if self.register_needing_key == None {
            // Turn off the draw flag, it gets set whenever we draw a sprite
            self.should_draw = false;
            // Get and execute the next opcode
            let op: u16 = self.get_op();
            self.execute_op(op);

            // The delay timer decrements every CPU cycle
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            // Each time the sound timer is decremented it triggers a beep
            if self.sound_timer > 0 {
                // TODO make some sound
                self.sound_timer -= 1;
            }
        }
        // TODO save state
    }

    /// Gets the opcode pointed to by the program_counter
    /// Interpreter memory is stored as bytes, but opcodes are 16 bits.
    /// Because of this we need to combine subsequent bytes.
    fn get_op(&self) -> u16 {
        let left = u16::from(self.memory[self.pc as usize]);
        let right = u16::from(self.memory[self.pc as usize + 1]);
        left << 8 | right
    }

    /// Draw a sprite on the display with wrapping.
    ///
    /// Sprites are XOR'ed onto the FrameBuffer, if this erases any pixels VF is set to 1 else 0.
    /// Sprites are 8 pixels wide by n pixels tall and are stored as n bytes.
    ///
    /// # Arguments
    /// * `x` - Vx is the x top left origin of the sprite
    /// * `y` - Vy is the y top left origin of the sprite
    /// * `n` - The sprite is read from bytes I..n
    fn draw_sprite(&mut self, x: u8, y: u8, n: u8) {
        self.should_draw = true;
        self.v[0xF] = 0;

        let sprite_x = self.v[x as usize];
        let sprite_y = self.v[y as usize];
        let sprite_data = &self.memory[(self.i as usize)..((self.i + n as u16) as usize)];

        // x/y dimensions of the display to handle wrapping
        let x_size = self.frame_buffer.len();
        let y_size = self.frame_buffer[0].len();

        for (y_idx, row_data) in sprite_data.iter().enumerate() {
            for x_idx in (0..8).rev() {
                let pixel_value: u8 = (row_data >> x_idx) as u8 & 0x1;
                let pixel_x: usize = (sprite_x as usize + x_idx) % x_size;
                let pixel_y: usize = (sprite_y as usize + y_idx) % y_size;

                let old_value = self.frame_buffer[pixel_x][pixel_y];
                self.frame_buffer[pixel_x][pixel_y] ^= pixel_value;

                if self.frame_buffer[pixel_x][pixel_y] != old_value {
                    self.v[0xF] = 1;
                }
            }
        }
    }

    // TODO refactor this to eliminate some repitition
    // TODO double check which opcodes should(n't) increment the pc
    /// Execute a single opcode
    fn execute_op(&mut self, op: u16) {
        // How much to increment the pc after executing this op
        let mut pc_increment: u16 = 2;
        match op.as_nibbles() {
            (0x0, 0x0, 0xE, 0x0) => {
                // Clear the display
                println!("CLS");
                self.frame_buffer = [[0; 32]; 64];
            }
            (0x0, 0x0, 0xE, 0xE) => {
                // Return from subroutine
                println!("RET");
                self.pc = self.stack[self.sp as usize];
                self.sp -= 0x1;
                // Don't increment the pc this cycle
                pc_increment = 0;
            }
            (0x1, ..) => {
                // Jump to addr
                let addr = op & 0x0FFF;
                println!("JP addr {:X}", addr);
                self.pc = addr;
                // Don't increment the pc this cycle
                pc_increment = 0;
            }
            (0x2, ..) => {
                // Call addr
                let addr = op & 0x0FFF;
                println!("CALL addr {:X}", addr);
                self.sp += 0x1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = addr;
                // Don't increment the pc this cycle
                pc_increment = 0;
            }
            (0x3, x, ..) => {
                // Skip next instruction if Vx == kk
                let kk = (op & 0x00FF) as u8;
                println!("SE Vx {:X}, {:X}", x, kk);
                if self.v[x as usize] == kk {
                    self.pc += 0x2;
                };
            }
            (0x4, x, ..) => {
                // Skip next instruction if Vx != kk
                let kk = (op & 0x00FF) as u8;
                println!("SE Vx {:X}, {:X}", x, kk);
                if self.v[x as usize] != kk {
                    self.pc += 0x2;
                };
            }
            (0x5, x, y, 0) => {
                // Skip next instruction if Vx == Vy
                println!("SE Vx {:X}, Vy {:X}", x, y);
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 0x2;
                };
            }
            (0x6, x, ..) => {
                // Vx = kk
                let kk = (op & 0x00FF) as u8;
                println!("LD Vx {:X}, {:X}", x, kk);
                self.v[x as usize] = kk;
            }
            (0x7, x, ..) => {
                // Vx += kk
                let kk = (op & 0x00FF) as u8;
                println!("Add Vx {:X}, {:X}", x, kk);
                self.v[x as usize] += kk;
            }
            (0x8, x, y, 0x0) => {
                // Vx = Vy
                println!("LD Vx {:X}, Vy {:X}", x, y);
                self.v[x as usize] = self.v[y as usize];
            }
            (0x8, x, y, 0x1) => {
                // Vx = Vx OR Vy
                println!("OR Vx {:X}, Vy {:X}", x, y);
                self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
            }
            (0x8, x, y, 0x2) => {
                // Vx = Vx AND Vy
                println!("AND Vx {:X}, Vy {:X}", x, y);
                self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
            }
            (0x8, x, y, 0x3) => {
                // Vx = Vx XOR Vy
                println!("XOR Vx {:X}, Vy {:X}", x, y);
                self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
            }
            (0x8, x, y, 0x4) => {
                // Vx = Vx + Vy
                // VF = overflow
                println!("ADD Vx {:X}, Vy {:X}", x, y);
                let (result, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[0xF] = if overflow { 0x1 } else { 0x0 };
                self.v[x as usize] = result;
            }
            (0x8, x, y, 0x5) => {
                // Vx = Vx - Vy
                // VF = not borrow
                println!("SUB Vx {:X}, Vy {:X}", x, y);
                let (result, overflow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[0xF] = if !overflow { 0x1 } else { 0x0 };
                self.v[x as usize] = result;
            }
            (0x8, x, _, 0x6) => {
                // VF = Vx least significant bit == 1
                // Vx /= 2
                println!("SHR Vx {:X}", x);
                self.v[0xF] = self.v[x as usize] & 0x1;
                self.v[x as usize] /= 0x2;
            }
            (0x8, x, y, 0x7) => {
                // Vx = Vy - Vx
                // Vx = not borrow
                println!("SUBN Vx {:X}, Vy {:X}", x, y);
                let (result, overflow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[0xF] = if overflow { 0x1 } else { 0x0 };
                self.v[x as usize] = result;
            }
            (0x8, x, _, 0xE) => {
                // VF = Vx least significant bit == 1
                // Vx *= Vx
                println!("SHL Vx {:X}", x);
                self.v[0xF] = self.v[x as usize] & 0x1;
                self.v[x as usize] *= 0x2;
            }
            (0x9, x, y, 0x0) => {
                // Skip next instruction if Vx != Vy
                println!("SNE Vx {:X}", x);
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 0x2
                };
            }
            (0xA, ..) => {
                // Set address register to addr
                let addr = op & 0x0FFF;
                println!("LD I addr {:X}", addr);
                self.i = addr;
            }
            (0xB, ..) => {
                // Set program counter to V0 + addr
                let addr = op & 0x0FFF;
                println!("JP V0 addr {:X}", addr);
                self.pc = self.v[0x0] as u16 + addr;
                // Don't increment the pc this cycle
                pc_increment = 0;
            }
            (0xC, x, ..) => {
                let kk = (op & 0x00FF) as u8;
                println!("RND Vx {:X}, {:X}", x, kk);
                let rand_byte: u8 = rand::random();
                self.v[x as usize] = rand_byte + kk;
            }
            (0xD, x, y, n) => {
                println!("DRW Vx {}, Vy {}, nibble {}", x, y, n);
                self.draw_sprite(x, y, n);
            }
            (0xE, x, 0x9, 0xE) => {
                // Skip the next instruction if key Vx is pressed
                println!("SKP Vx {}", x);
                if self.pressed_keys[self.v[x as usize] as usize] == 0x1 {
                    self.pc += 2;
                };
            }
            (0xE, x, 0xA, 0x1) => {
                // Skip the next instruction if key Vx is not pressed
                println!("SKNP Vx {}", x);
                if self.pressed_keys[self.v[x as usize] as usize] == 0x0 {
                    self.pc += 2;
                };
            }
            (0xF, x, 0x0, 0x7) => {
                // Vx = delay_timer
                println!("LD Vx {:X}, DT", x);
                self.v[x as usize] = self.delay_timer;
            }
            (0xF, x, 0x0, 0xA) => {
                // Await keypress and store it in Vx
                println!("LD Vx {}, K", x);
                self.register_needing_key = Some(x)
            }
            (0xF, x, 0x1, 0x5) => {
                // delay_timer = Vx
                println!("LD DT, Vx {:X}", x);
                self.delay_timer = self.v[x as usize];
            }
            (0xF, x, 0x1, 0x8) => {
                // sound_timer = Vx
                println!("LD ST, Vx {:X}", x);
                self.sound_timer = self.v[x as usize];
            }
            (0xF, x, 0x1, 0xE) => {
                // I = I + Vx
                println!("ADD I, Vx {:X}", x);
                self.i += self.v[x as usize] as u16;
            }
            (0xF, x, 0x2, 0x9) => {
                // I = memory addr for the hexadecimal sprite Vx
                // See sprites::SPRITE_SHEET for more details
                println!("LD F, Vx {:X}", x);
                self.i = self.v[x as usize] as u16 * 5;
            }
            (0xF, x, 0x3, 0x3) => {
                // Store BCD repr of Vx in memory starting at address i
                println!("LD B, Vx {:X}", x);
                let bcd = [
                    (self.v[x as usize] / 100 % 10),
                    (self.v[x as usize] / 10 % 10),
                    (self.v[x as usize] % 10),
                ];
                self.memory[self.i as usize..(self.i + 0x3) as usize].copy_from_slice(&bcd);
            }
            (0xF, x, 0x5, 0x5) => {
                // Fill memory starting at address i with V0..Vx
                println!("LD [I], Vx {:X}", x);
                self.memory[self.i as usize..(self.i + x as u16) as usize]
                    .copy_from_slice(&self.v[0x0 as usize..x as usize]);
            }
            (0xF, x, 0x6, 0x5) => {
                // Fill V0..Vx with memory starting at address i
                println!("LD Vx {:X}, [I]", x);
                self.v[0x0 as usize..x as usize]
                    .copy_from_slice(&self.memory[self.i as usize..(self.i + x as u16) as usize]);
            }
            other => panic!("Opcode {:?} is not implemented", other),
        }
        self.pc += pc_increment;
    }
}
