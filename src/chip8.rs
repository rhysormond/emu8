use std::collections::VecDeque;

use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, MAX_SAVED_STATES};
use opcode::Opcode;
use state::{FrameBuffer, State};

/// # Chip-8
/// Chip-8 is a virtual machine and corresponding interpreted language.
/// Tracks current state as well as past states for the purposes of rewinding.
///
/// Is interfaced with by the outside world via methods to:
/// - load roms
/// - press and release keys
/// - advance and reverse CPU cycles
/// - advance its timers
/// - inspect its frame buffer for rendering by some display
pub struct Chip8 {
    state: State,
    previous_states: VecDeque<State>,
}

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
    ///
    /// # Arguments
    /// * `key` the 8-bit representation of the key that was pressed
    pub fn key_press(&mut self, key: u8) {
        self.state.pressed_keys[key as usize] = 0x1;
        if let Some(register) = self.state.register_needing_key {
            self.state.v[register as usize] = key;
            self.state.register_needing_key = None;
        }
    }

    /// Unset the pressed status of key
    ///
    /// # Arguments
    /// * `key` the 8-bit representation of the key that was released
    pub fn key_release(&mut self, key: u8) {
        self.state.pressed_keys[key as usize] = 0x0;
    }

    /// Advances the CPU by a single cycle
    /// - breaks if awaiting a keypress
    /// - gets and executes the next opcode
    pub fn advance_cycle(&mut self) {
        if self.state.register_needing_key == None {
            let op: u16 = self.get_op();
            self.state = op.execute(&self.state);
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
        // TODO this should be foreach or something similar since it has a unit return
        self.previous_states
            .pop_front()
            .map(|state| self.state = state);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip8_gets_op() {
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
        chip8.save_state();
        assert_eq!(chip8.previous_states.len(), 1);
    }

    #[test]
    fn test_chip8_drops_old_saved_states() {
        let mut chip8 = Chip8::new();
        for _ in 0..MAX_SAVED_STATES {
            chip8.save_state();
        }
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
