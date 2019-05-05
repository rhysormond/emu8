use constants::*;
use state::*;

/// # Opcodes
///
/// Chip-8 opcodes are 16 bits each. Their behavior is cased on some combination of:
/// - `(n, _, _, _)` broad categorization; applies to all opcodes
/// - `(_, _, _, n)` specific behavior within a category
/// - `(_, _, n, n)` more specific behavior within a category
/// - `(_, n, n, n)` some fixed function that doesn't require variables (e.g. CLS; clear screen)
///
/// Nibbles not used to determine the operation often (but not always) carry important data.
/// - `(_, n, n, n)` represent a 16-bit address
/// - `(_, _, n, n)` encodes some data that is assigned to and/or compared with Vx
/// - `(_, n, _, _)` refers either to the register Vx or a range of registers V0..Vx
/// - `(_, _, n, _)` refers to the the register Vy
pub trait Opcode
where
    Self: Sized,
{
    /// Returns the Opcode's component nibbles.
    fn nibbles(&self) -> (u8, u8, u8, u8);

    /// Returns the Opcode's without its most significant nibble.
    fn addr(&self) -> u16;

    /// Returns the Opcode's least significant byte.
    fn byte(&self) -> u8;

    /// Executes an opcode on a given state and returns an updated copy of it
    fn execute(&self, state: &State) -> State;
}

impl Opcode for u16 {
    fn nibbles(&self) -> (u8, u8, u8, u8) {
        (
            ((self & 0xF000) >> 12) as u8,
            ((self & 0x0F00) >> 8) as u8,
            ((self & 0x00F0) >> 4) as u8,
            (self & 0x000F) as u8,
        )
    }

    fn addr(&self) -> u16 {
        self & 0x0FFF
    }

    fn byte(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    /// Execute a single opcode
    ///
    /// Matches the opcode's nibbles against a table and use them to conditionally edit the state.
    /// Returns a copy of the state after it's transformed by the opcode execution.
    ///
    /// # Arguments
    /// * `op` a 16-bit opcode
    /// * `state` a reference to the Chip-8's internal state
    fn execute(&self, state: &State) -> State {
        let mut state: State = State::clone(state);
        // TODO refactor this to eliminate some repetition
        // TODO use a logger instead of print statements
        print!("{:04X} ", self);

        // how much to increment pc after executing the op
        let mut pc_bump: u16 = 0x2;
        match self.nibbles() {
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
                let addr = self.addr();
                println!("JP   | PC = {:04X}", addr);
                state.pc = addr;
                pc_bump = 0x0;
            }
            (0x2, ..) => {
                let addr = self.addr();
                println!("CALL | STACK.push(PC); PC = {:04X}", addr);
                state.sp += 0x1;
                state.stack[state.sp as usize] = state.pc;
                state.pc = addr;
                pc_bump = 0x0;
            }
            (0x3, x, ..) => {
                let kk = self.byte();
                println!("SE   | if V{:X} == {:X} pc += 2", x, kk);
                if state.v[x as usize] == kk {
                    state.pc += 0x2;
                };
            }
            (0x4, x, ..) => {
                let kk = self.byte();
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
                let kk = self.byte();
                println!("LD   | V{:X} = {:X}", x, kk);
                state.v[x as usize] = kk;
            }
            (0x7, x, ..) => {
                let kk = self.byte();
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
                let addr = self.addr();
                println!("LD   | I = {:04X}", addr);
                state.i = addr;
            }
            (0xB, ..) => {
                let addr = self.addr();
                println!("JP   | PC = V0 + {:04X}", addr);
                state.pc = state.v[0x0] as u16 + addr;
                pc_bump = 0x0;
            }
            (0xC, x, ..) => {
                let kk = self.byte();
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
mod test_opcode {
    use super::*;

    #[test]
    fn test_nibbles() {
        let op: u16 = 0xABCD;
        assert_eq!(op.nibbles(), (0xA, 0xB, 0xC, 0xD));
    }

    #[test]
    fn test_addr() {
        let op: u16 = 0xABCD;
        assert_eq!(op.addr(), 0x0BCD);
    }

    #[test]
    fn test_byte() {
        let op: u16 = 0xABCD;
        assert_eq!(op.byte(), 0x00CD);
    }
}

#[cfg(test)]
mod test_execute {
    use super::*;

    #[test]
    fn test_00e0_cls() {
        let mut state = State::new();
        state.frame_buffer[0][0] = 1;
        let state = 0x00E0.execute(&state);
        assert_eq!(state.frame_buffer[0][0], 0);
    }

    #[test]
    fn test_00ee_ret() {
        let mut state = State::new();
        state.sp = 0x1;
        state.stack[state.sp as usize] = 0xABCD;
        let state = 0x00EE.execute(&state);
        assert_eq!(state.sp, 0x0);
        // Add 2 to the program as it's bumped after opcode execution
        assert_eq!(state.pc, 0xABCD + 0x2);
    }

    #[test]
    fn test_1nnn_jp() {
        let state = State::new();
        let state = 0x1ABC.execute(&state);
        assert_eq!(state.pc, 0x0ABC);
    }

    #[test]
    fn test_2nnn_call() {
        let mut state = State::new();
        state.pc = 0xABCD;
        let state = 0x2123.execute(&state);
        assert_eq!(state.sp, 0x1);
        assert_eq!(state.stack[state.sp as usize], 0xABCD);
        assert_eq!(state.pc, 0x0123);
    }

    #[test]
    fn test_3xkk_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x3111.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_se_doesntskip() {
        let state = State::new();
        let state = 0x3111.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_4xkk_sne_skips() {
        let state = State::new();
        let state = 0x4111.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x4111.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_5xy0_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = 0x5120.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_5xy0_se_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x5120.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_6xkk_ld() {
        let state = State::new();
        let state = 0x6122.execute(&state);
        assert_eq!(state.v[0x1], 0x22);
    }

    #[test]
    fn test_7xkk_add() {
        let mut state = State::new();
        state.v[0x1] = 0x1;
        let state = 0x7122.execute(&state);
        assert_eq!(state.v[0x1], 0x23);
    }

    #[test]
    fn test_8xy0_ld() {
        let mut state = State::new();
        state.v[0x2] = 0x1;
        let state = 0x8120.execute(&state);
        assert_eq!(state.v[0x1], 0x1);
    }

    #[test]
    fn test_8xy1_or() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8121.execute(&state);
        assert_eq!(state.v[0x1], 0x7);
    }

    #[test]
    fn test_8xy2_and() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8122.execute(&state);
        assert_eq!(state.v[0x1], 0x2);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8123.execute(&state);
        assert_eq!(state.v[0x1], 0x5);
    }

    #[test]
    fn test_8xy4_add_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0xEE;
        state.v[0x2] = 0x11;
        let state = 0x8124.execute(&state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        state.v[0x2] = 0x11;
        let state = 0x8124.execute(&state);
        assert_eq!(state.v[0x1], 0x10);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x33;
        state.v[0x2] = 0x11;
        let state = 0x8125.execute(&state);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x12;
        let state = 0x8125.execute(&state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy6_shr_lsb() {
        let mut state = State::new();
        state.v[0x1] = 0x5;
        let state = 0x8106.execute(&state);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy6_shr_nolsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = 0x8106.execute(&state);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy7_subn_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x33;
        let state = 0x8127.execute(&state);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy7_subn_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x12;
        state.v[0x2] = 0x11;
        let state = 0x8127.execute(&state);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xye_shl_msb() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        let state = 0x810E.execute(&state);
        // 0xFF * 2 = 0x01FE
        assert_eq!(state.v[0x1], 0xFE);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xye_shl_nomsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = 0x810E.execute(&state);
        assert_eq!(state.v[0x1], 0x8);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_9xy0_sne_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x9120.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_9xy0_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = 0x9120.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_annn_ld() {
        let state = State::new();
        let state = 0xAABC.execute(&state);
        assert_eq!(state.i, 0xABC);
    }

    #[test]
    fn test_bnnn_jp() {
        let mut state = State::new();
        state.v[0x0] = 0x2;
        let state = 0xBABC.execute(&state);
        assert_eq!(state.pc, 0xABE);
    }

    // Not testing cxkk as it generates a random number

    #[test]
    fn test_dxyn_drw_draws() {
        let mut state = State::new();
        state.v[0x0] = 0x1;
        // Draw the 0x0 sprite with a 1x 1y offset
        let state = 0xD005.execute(&state);
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
        let state = 0xD001.execute(&state);
        assert_eq!(state.v[0xF], 0x1)
    }

    #[test]
    fn test_dxyn_drw_xors() {
        let mut state = State::new();
        // 0 1 0 1 -> Set
        state.frame_buffer[0][2..6].copy_from_slice(&[0, 1, 0, 1]);
        // 1 1 0 0 -> Draw xor
        let state = 0xD005.execute(&state);
        assert_eq!(state.frame_buffer[0][2..6], [1, 0, 0, 1])
    }

    #[test]
    fn test_ex9e_skp_skips() {
        let mut state = State::new();
        state.pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = 0xE19E.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_ex9e_skp_doesntskip() {
        let state = State::new();
        let state = 0xE19E.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_exa1_sknp_skips() {
        let state = State::new();
        let state = 0xE1A1.execute(&state);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_exa1_sknp_doesntskip() {
        let mut state = State::new();
        state.pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = 0xE1A1.execute(&state);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_fx07_ld() {
        let mut state = State::new();
        state.delay_timer = 0xF;
        let state = 0xF107.execute(&state);
        assert_eq!(state.v[0x1], 0xF);
    }

    #[test]
    fn test_fx0a_ld_setsregisterneedingkey() {
        let state = State::new();
        let state = 0xF10A.execute(&state);
        assert_eq!(state.register_needing_key, Some(0x1));
    }

    #[test]
    fn test_fx15_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = 0xf115.execute(&state);
        assert_eq!(state.delay_timer, 0xF);
    }

    #[test]
    fn test_fx18_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = 0xf118.execute(&state);
        assert_eq!(state.sound_timer, 0xF);
    }

    #[test]
    fn test_fx1e_add() {
        let mut state = State::new();
        state.i = 0x1;
        state.v[0x1] = 0x1;
        let state = 0xF11E.execute(&state);
        assert_eq!(state.i, 0x2);
    }

    #[test]
    fn test_fx29_ld() {
        let mut state = State::new();
        state.v[0x1] = 0x2;
        let state = 0xF129.execute(&state);
        assert_eq!(state.i, 0xA);
    }

    #[test]
    fn test_fx33_ld() {
        let mut state = State::new();
        // 0x7B -> 123
        state.v[0x1] = 0x7B;
        state.i = 0x200;
        let state = 0xF133.execute(&state);
        assert_eq!(state.memory[0x200..0x203], [0x1, 0x2, 0x3]);
    }

    #[test]
    fn test_fx_55_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.v[0x0..0x5].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = 0xF455.execute(&state);
        assert_eq!(state.memory[0x200..0x205], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }

    #[test]
    fn test_fx_65_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.memory[0x200..0x205].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = 0xF465.execute(&state);
        assert_eq!(state.v[0x0..0x5], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }
}
