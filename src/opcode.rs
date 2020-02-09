use instruction::*;

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
    /// TODO[Rhys] this should probably be expressed as From/Into
    fn addr(&self) -> u16;

    /// Returns the Opcode's least significant byte.
    /// TODO[Rhys] this should probably be expressed as From/Into
    fn byte(&self) -> u8;

    /// Returns the instruction that corresponds to the opcode
    fn to_instruction(&self) -> Box<dyn Instruction>;
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

    fn to_instruction(&self) -> Box<dyn Instruction> {
        match self.nibbles() {
            (0x0, 0x0, 0xE, 0x0) => Box::new(Clr),
            (0x0, 0x0, 0xE, 0xE) => Box::new(Rts),
            (0x1, ..) => Box::new(Jump { addr: self.addr() }),
            (0x2, ..) => Box::new(Call { addr: self.addr() }),
            (0x3, x, ..) => Box::new(Ske { x, kk: self.byte() }),
            (0x4, x, ..) => Box::new(Skne { x, kk: self.byte() }),
            (0x5, x, y, 0x0) => Box::new(Skre { x, y }),
            (0x6, x, ..) => Box::new(Load { x, kk: self.byte() }),
            (0x7, x, ..) => Box::new(Add { x, kk: self.byte() }),
            (0x8, x, y, 0x0) => Box::new(Move { x, y }),
            (0x8, x, y, 0x1) => Box::new(Or { x, y }),
            (0x8, x, y, 0x2) => Box::new(And { x, y }),
            (0x8, x, y, 0x3) => Box::new(Xor { x, y }),
            (0x8, x, y, 0x4) => Box::new(Addr { x, y }),
            (0x8, x, y, 0x5) => Box::new(Sub { x, y }),
            (0x8, x, _, 0x6) => Box::new(Shr { x }),
            (0x8, x, y, 0x7) => Box::new(Subn { x, y }),
            (0x8, x, _, 0xE) => Box::new(Shl { x }),
            (0x9, x, y, 0x0) => Box::new(Skrne { x, y }),
            (0xA, ..) => Box::new(Loadi { addr: self.addr() }),
            (0xB, ..) => Box::new(Jumpi { addr: self.addr() }),
            (0xC, x, ..) => Box::new(Rand { x, kk: self.byte() }),
            (0xD, x, y, n) => Box::new(Draw { x, y, n }),
            (0xE, x, 0x9, 0xE) => Box::new(Skpr { x }),
            (0xE, x, 0xA, 0x1) => Box::new(Skup { x }),
            (0xF, x, 0x0, 0x7) => Box::new(Moved { x }),
            (0xF, x, 0x0, 0xA) => Box::new(Keyd { x }),
            (0xF, x, 0x1, 0x5) => Box::new(Loads { x }),
            (0xF, x, 0x1, 0x8) => Box::new(Ld { x }),
            (0xF, x, 0x1, 0xE) => Box::new(Addi { x }),
            (0xF, x, 0x2, 0x9) => Box::new(Ldspr { x }),
            (0xF, x, 0x3, 0x3) => Box::new(Bcd { x }),
            (0xF, x, 0x5, 0x5) => Box::new(Stor { x }),
            (0xF, x, 0x6, 0x5) => Box::new(Read { x }),
            other => panic!("Opcode {:?} is not implemented", other),
        }
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
mod test_instruction {
    use super::*;
    use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
    use state::State;

    #[test]
    fn test_00e0_cls() {
        let mut state = State::new();
        state.frame_buffer[0][0] = 1;
        let state = 0x00E0.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.frame_buffer[0][0], 0);
    }

    #[test]
    fn test_00ee_ret() {
        let mut state = State::new();
        state.sp = 0x1;
        state.stack[state.sp as usize] = 0xABCD;
        let state = 0x00EE.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.sp, 0x0);
        // Add 2 to the program as it's bumped after opcode execution
        assert_eq!(state.pc, 0xABCD + 0x2);
    }

    #[test]
    fn test_1nnn_jp() {
        let state = State::new();
        let state = 0x1ABC.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0ABC);
    }

    #[test]
    fn test_2nnn_call() {
        let mut state = State::new();
        state.pc = 0xABCD;
        let state = 0x2123.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.sp, 0x1);
        assert_eq!(state.stack[state.sp as usize], 0xABCD);
        assert_eq!(state.pc, 0x0123);
    }

    #[test]
    fn test_3xkk_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x3111.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_se_doesntskip() {
        let state = State::new();
        let state = 0x3111.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_4xkk_sne_skips() {
        let state = State::new();
        let state = 0x4111.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_3xkk_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x4111.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_5xy0_se_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = 0x5120.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_5xy0_se_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x5120.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_6xkk_ld() {
        let state = State::new();
        let state = 0x6122.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x22);
    }

    #[test]
    fn test_7xkk_add() {
        let mut state = State::new();
        state.v[0x1] = 0x1;
        let state = 0x7122.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x23);
    }

    #[test]
    fn test_8xy0_ld() {
        let mut state = State::new();
        state.v[0x2] = 0x1;
        let state = 0x8120.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x1);
    }

    #[test]
    fn test_8xy1_or() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8121.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x7);
    }

    #[test]
    fn test_8xy2_and() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8122.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x2);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut state = State::new();
        state.v[0x1] = 0x6;
        state.v[0x2] = 0x3;
        let state = 0x8123.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x5);
    }

    #[test]
    fn test_8xy4_add_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0xEE;
        state.v[0x2] = 0x11;
        let state = 0x8124.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        state.v[0x2] = 0x11;
        let state = 0x8124.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x10);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x33;
        state.v[0x2] = 0x11;
        let state = 0x8125.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy5_sub_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x12;
        let state = 0x8125.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy6_shr_lsb() {
        let mut state = State::new();
        state.v[0x1] = 0x5;
        let state = 0x8106.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy6_shr_nolsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = 0x8106.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x2);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xy7_subn_nocarry() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x33;
        let state = 0x8127.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x22);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xy7_subn_carry() {
        let mut state = State::new();
        state.v[0x1] = 0x12;
        state.v[0x2] = 0x11;
        let state = 0x8127.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0xFF);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_8xye_shl_msb() {
        let mut state = State::new();
        state.v[0x1] = 0xFF;
        let state = 0x810E.to_instruction().execute(&state, [0; 16]);
        // 0xFF * 2 = 0x01FE
        assert_eq!(state.v[0x1], 0xFE);
        assert_eq!(state.v[0xF], 0x1);
    }

    #[test]
    fn test_8xye_shl_nomsb() {
        let mut state = State::new();
        state.v[0x1] = 0x4;
        let state = 0x810E.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0x8);
        assert_eq!(state.v[0xF], 0x0);
    }

    #[test]
    fn test_9xy0_sne_skips() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        let state = 0x9120.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_9xy0_sne_doesntskip() {
        let mut state = State::new();
        state.v[0x1] = 0x11;
        state.v[0x2] = 0x11;
        let state = 0x9120.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_annn_ld() {
        let state = State::new();
        let state = 0xAABC.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.i, 0xABC);
    }

    #[test]
    fn test_bnnn_jp() {
        let mut state = State::new();
        state.v[0x0] = 0x2;
        let state = 0xBABC.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0xABE);
    }

    // Not testing cxkk as it generates a random number

    #[test]
    fn test_dxyn_drw_draws() {
        let mut state = State::new();
        state.v[0x0] = 0x1;
        // Draw the 0x0 sprite with a 1x 1y offset
        let state = 0xD005.to_instruction().execute(&state, [0; 16]);
        let mut expected = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
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
        let state = 0xD001.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0xF], 0x1)
    }

    #[test]
    fn test_dxyn_drw_xors() {
        let mut state = State::new();
        // 0 1 0 1 -> Set
        state.frame_buffer[0][2..6].copy_from_slice(&[0, 1, 0, 1]);
        // 1 1 0 0 -> Draw xor
        let state = 0xD005.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.frame_buffer[0][2..6], [1, 0, 0, 1])
    }

    #[test]
    fn test_ex9e_skp_skips() {
        let mut state = State::new();
        let mut pressed_keys = [0; 16];
        pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = 0xE19E.to_instruction().execute(&state, pressed_keys);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_ex9e_skp_doesntskip() {
        let state = State::new();
        let state = 0xE19E.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_exa1_sknp_skips() {
        let state = State::new();
        let state = 0xE1A1.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.pc, 0x0204);
    }

    #[test]
    fn test_exa1_sknp_doesntskip() {
        let mut state = State::new();
        let mut pressed_keys = [0; 16];
        pressed_keys[0xE] = 0x1;
        state.v[0x1] = 0xE;
        let state = 0xE1A1.to_instruction().execute(&state, pressed_keys);
        assert_eq!(state.pc, 0x0202);
    }

    #[test]
    fn test_fx07_ld() {
        let mut state = State::new();
        state.delay_timer = 0xF;
        let state = 0xF107.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x1], 0xF);
    }

    #[test]
    fn test_fx0a_ld_setsregisterneedingkey() {
        let state = State::new();
        let state = 0xF10A.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.register_needing_key, Some(0x1));
    }

    #[test]
    fn test_fx15_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = 0xf115.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.delay_timer, 0xF);
    }

    #[test]
    fn test_fx18_ld() {
        let mut state = State::new();
        state.v[0x1] = 0xF;
        let state = 0xf118.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.sound_timer, 0xF);
    }

    #[test]
    fn test_fx1e_add() {
        let mut state = State::new();
        state.i = 0x1;
        state.v[0x1] = 0x1;
        let state = 0xF11E.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.i, 0x2);
    }

    #[test]
    fn test_fx29_ld() {
        let mut state = State::new();
        state.v[0x1] = 0x2;
        let state = 0xF129.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.i, 0xA);
    }

    #[test]
    fn test_fx33_ld() {
        let mut state = State::new();
        // 0x7B -> 123
        state.v[0x1] = 0x7B;
        state.i = 0x200;
        let state = 0xF133.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.memory[0x200..0x203], [0x1, 0x2, 0x3]);
    }

    #[test]
    fn test_fx_55_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.v[0x0..0x5].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = 0xF455.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.memory[0x200..0x205], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }

    #[test]
    fn test_fx_65_ld() {
        let mut state = State::new();
        state.i = 0x200;
        state.memory[0x200..0x205].copy_from_slice(&[0x1, 0x2, 0x3, 0x4, 0x5]);
        let state = 0xF465.to_instruction().execute(&state, [0; 16]);
        assert_eq!(state.v[0x0..0x5], [0x1, 0x2, 0x3, 0x4, 0x5]);
    }
}
