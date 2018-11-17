extern crate sdl2;

mod cpu;

pub use chip8::cpu::CPU;

use sdl2::keyboard::Keycode;

/// # Chip-8 FrameBuffer
/// The frame buffer stores the contents of a single Chip-8 frame.
pub type FrameBuffer = [[u8; 32]; 64];

/// # Chip-8 ROM
/// Some ROM file to be loaded by the Chip-8.
pub type ROM = [u8; 4096];

/// # Chip-8
/// Chip-8 is a virtual machine and corresponding interpreted language.
///
/// # Memory
/// - 64 byte stack to store return addresses when subroutines are called
/// - 32x64 frame buffer to store the next frame to be drawn by the Display
/// - 4096 bytes of addressable memory
/// - optionally a ROM
pub struct Chip8 {
    cpu: CPU,
    stack: [u8; 16],
    pub frame_buffer: FrameBuffer,
    memory: [u8; 4096],
    rom: Option<ROM>,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            cpu: CPU::new(),
            stack: [0; 16],
            frame_buffer: [[0; 32]; 64],
            memory: [0; 4096],
            rom: None,
        }
    }

    /// Handles input from event loop.
    pub fn input(&mut self, _key: Keycode) {
        // TODO Actually handle input
    }

    /// Execute one CPU cycle.
    pub fn cycle(&mut self) {
        // TODO Decrement Delay Timer
        // TODO Decrement Sound Timer
        // TODO Increment Program Counter if it needs it
    }
}
