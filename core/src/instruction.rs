use crate::constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::state::State;

/// Chip8 instructions that know how to execute themselves
pub trait Instruction {
    /// Execute the instruction and return an updated state
    ///
    /// NOTE: while some opcodes interact with the set of pressed keys, a lot of the keypress
    ///       interaction happens when the key itself is pressed (see `Chip8.key_press`)
    ///
    /// # Arguments
    /// * `state` a reference to the Chip-8's internal state
    /// * `pressed_keys` the currently pressed keys
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State;
}

/// clear
pub struct Clr;

impl Instruction for Clr {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            frame_buffer: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            draw_flag: true,
            ..*state
        }
    }
}

/// PC = STACK.pop()
pub struct Rts;

impl Instruction for Rts {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.stack[state.sp as usize] + 0x2,
            sp: state.sp - 0x1,
            ..*state
        }
    }
}

// PC = addr
pub struct Jump {
    pub addr: u16,
}

impl Instruction for Jump {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: self.addr,
            ..*state
        }
    }
}

/// STACK.push(PC); PC = addr
pub struct Call {
    pub addr: u16,
}

impl Instruction for Call {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut sp = state.sp;
        sp += 0x1;
        let mut stack = state.stack;
        stack[sp as usize] = state.pc;
        State {
            pc: self.addr,
            sp,
            stack,
            ..*state
        }
    }
}

/// if Vx == kk then pc += 2
pub struct Ske {
    pub x: u8,
    pub kk: u8,
}

impl Instruction for Ske {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let pc = if state.v[self.x as usize] == self.kk {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// if Vx != kk then pc += 2
pub struct Skne {
    pub x: u8,
    pub kk: u8,
}

impl Instruction for Skne {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let pc = if state.v[self.x as usize] != self.kk {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// if Vx == Vy then pc += 2
pub struct Skre {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Skre {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let pc = if state.v[self.x as usize] == state.v[self.y as usize] {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// Vx = kk
pub struct Load {
    pub x: u8,
    pub kk: u8,
}

impl Instruction for Load {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] = self.kk;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx += kk
/// Add kk to Vx; allow for overflow but implicitly drop it
pub struct Add {
    pub x: u8,
    pub kk: u8,
}

impl Instruction for Add {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let (res, _) = state.v[self.x as usize].overflowing_add(self.kk);
        let mut v = state.v;
        v[self.x as usize] = res;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx = Vy
pub struct Move {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Move {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] = v[self.y as usize];
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx |= Vy
pub struct Or {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Or {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] |= v[self.y as usize];
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx &= Vy
pub struct And {
    pub x: u8,
    pub y: u8,
}

impl Instruction for And {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] &= v[self.y as usize];
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx ^= Vy
pub struct Xor {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Xor {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] ^= v[self.y as usize];
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx += Vy; VF = overflow
pub struct Addr {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Addr {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let (res, over) = state.v[self.x as usize].overflowing_add(state.v[self.y as usize]);
        let mut v = state.v;
        v[0xF] = if over { 0x1 } else { 0x0 };
        v[self.x as usize] = res;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx -= Vy; VF = !underflow
pub struct Sub {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Sub {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let (res, under) = state.v[self.x as usize].overflowing_sub(state.v[self.y as usize]);
        let mut v = state.v;
        v[0xF] = if under { 0x0 } else { 0x1 };
        v[self.x as usize] = res;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx /= 2; VF = underflow
pub struct Shr {
    pub x: u8,
}

impl Instruction for Shr {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[0xF] = v[self.x as usize] & 0x1;
        v[self.x as usize] /= 0x2;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx -= Vy; VF = underflow
pub struct Subn {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Subn {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let (res, under) = state.v[self.y as usize].overflowing_sub(state.v[self.x as usize]);
        let mut v = state.v;
        v[0xF] = if under { 0x0 } else { 0x1 };
        v[self.x as usize] = res;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// Vx *= 2; VF = overflow
pub struct Shl {
    pub x: u8,
}

impl Instruction for Shl {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let (res, over) = state.v[self.x as usize].overflowing_mul(2);
        let mut v = state.v;
        v[0xF] = if over { 0x1 } else { 0x0 };
        v[self.x as usize] = res;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// if Vx != Vy then pc +=2
pub struct Skrne {
    pub x: u8,
    pub y: u8,
}

impl Instruction for Skrne {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let pc = if state.v[self.x as usize] != state.v[self.y as usize] {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// I = addr
pub struct Loadi {
    pub addr: u16,
}

impl Instruction for Loadi {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            i: self.addr,
            ..*state
        }
    }
}

/// PC = V0 + addr
pub struct Jumpi {
    pub addr: u16,
}

impl Instruction for Jumpi {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: u16::from(state.v[0x0]) + self.addr,
            ..*state
        }
    }
}

/// Vx = rand_byte + kk
pub struct Rand {
    pub x: u8,
    pub kk: u8,
}

impl Instruction for Rand {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let rand_byte: u8 = rand::random();
        let mut v = state.v;
        v[self.x as usize] = rand_byte & self.kk;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// draw_sprite(x=Vx y=Vy size=n)
/// XORs a sprite from memory i..n at position x, y on the FrameBuffer with wrapping.
/// Sets VF if any pixels would be erased
pub struct Draw {
    pub x: u8,
    pub y: u8,
    pub n: u8,
}

impl Instruction for Draw {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        let mut frame_buffer = state.frame_buffer;

        // Reset the carry flag (used for collision detection)
        v[0xF] = 0x0;

        for byte in 0..self.n as usize {
            let y = (state.v[self.y as usize] as usize + byte) % DISPLAY_HEIGHT;
            for bit in 0..8 {
                let x = (state.v[self.x as usize] as usize + bit) % DISPLAY_WIDTH;
                let pixel_value = (state.memory[state.i as usize + byte] >> (7 - bit) as u8) & 1;
                v[0xF] |= pixel_value & state.frame_buffer[y as usize][x as usize];
                frame_buffer[y as usize][x as usize] ^= pixel_value;
            }
        }

        State {
            pc: state.pc + 0x2,
            draw_flag: true,
            v,
            frame_buffer,
            ..*state
        }
    }
}

/// if Vx.pressed then pc += 2
pub struct Skpr {
    pub x: u8,
}

impl Instruction for Skpr {
    fn execute(&self, state: &State, pressed_keys: [u8; 16]) -> State {
        let pc = if pressed_keys[state.v[self.x as usize] as usize] == 0x1 {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// if !Vx.pressed then pc += 2
pub struct Skup {
    pub x: u8,
}

impl Instruction for Skup {
    fn execute(&self, state: &State, pressed_keys: [u8; 16]) -> State {
        let pc = if pressed_keys[state.v[self.x as usize] as usize] == 0x0 {
            state.pc + 0x4
        } else {
            state.pc + 0x2
        };
        State { pc, ..*state }
    }
}

/// Vx = DT
pub struct Moved {
    pub x: u8,
}

impl Instruction for Moved {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[self.x as usize] = state.delay_timer;
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}

/// await keypress for Vx
pub struct Keyd {
    pub x: u8,
}

impl Instruction for Keyd {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            register_needing_key: Some(self.x),
            ..*state
        }
    }
}

/// DT = Vx
pub struct Loads {
    pub x: u8,
}

impl Instruction for Loads {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            delay_timer: state.v[self.x as usize],
            ..*state
        }
    }
}

/// ST = Vx
pub struct Ld {
    pub x: u8,
}

impl Instruction for Ld {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            sound_timer: state.v[self.x as usize],
            ..*state
        }
    }
}

/// I += Vx
pub struct Addi {
    pub x: u8,
}

impl Instruction for Addi {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            i: state.i + u16::from(state.v[self.x as usize]),
            ..*state
        }
    }
}

/// I = Vx * 5
/// Set I to the memory address of the sprite for Vx
/// See sprites::SPRITE_SHEET for more details
pub struct Ldspr {
    pub x: u8,
}

impl Instruction for Ldspr {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        State {
            pc: state.pc + 0x2,
            i: u16::from(state.v[self.x as usize]) * 5,
            ..*state
        }
    }
}

/// mem[I..I+3] = bcd(Vx)
/// Store BCD repr of Vx in memory starting at address i
pub struct Bcd {
    pub x: u8,
}

impl Instruction for Bcd {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let bcd = [
            (state.v[self.x as usize] / 100 % 10),
            (state.v[self.x as usize] / 10 % 10),
            (state.v[self.x as usize] % 10),
        ];
        let mut memory = state.memory;
        memory[state.i as usize..(state.i + 0x3) as usize].copy_from_slice(&bcd);
        State {
            pc: state.pc + 0x2,
            memory,
            ..*state
        }
    }
}

/// mem[I..I+x] = V0..Vx
/// Fill memory starting at address i with V0..Vx+1
pub struct Stor {
    pub x: u8,
}

impl Instruction for Stor {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut memory = state.memory;
        memory[state.i as usize..=(state.i + u16::from(self.x)) as usize]
            .copy_from_slice(&state.v[0x0 as usize..=self.x as usize]);
        State {
            pc: state.pc + 0x2,
            memory,
            ..*state
        }
    }
}

/// V0..Vx = mem[I..I+x]
/// Fill V0..Vx+1 with memory starting at address i
pub struct Read {
    pub x: u8,
}

impl Instruction for Read {
    fn execute(&self, state: &State, _pressed_keys: [u8; 16]) -> State {
        let mut v = state.v;
        v[0x0 as usize..=self.x as usize].copy_from_slice(
            &state.memory[state.i as usize..=(state.i + u16::from(self.x)) as usize],
        );
        State {
            pc: state.pc + 0x2,
            v,
            ..*state
        }
    }
}
