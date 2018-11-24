extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

mod chip8;
mod display;
mod keymap;
mod opcode;
mod sprites;

fn main() {
    // TODO Error handling
    // Initializing State
    let sdl = sdl2::init().unwrap();
    let mut chip8 = chip8::Chip8::new();
    let mut display = display::Display::new(&sdl, 64, 32, 10);
    let mut events = sdl.event_pump().unwrap();
    // TODO Load ROMs

    // Timing (the Chip-8 has a frame_rate of 60Hz -> 16.7 milliseconds/frame)
    let frame_rate = Duration::new(0, 16_666_667);
    let mut last_frame = Instant::now();
    // TODO Log stuff to see what's actually going on
    'event: loop {
        if chip8.should_draw == true {
            // Get the state of the Chip-8 FrameBuffer and draw it
            display.render_frame(&chip8.frame_buffer);
        }

        // Check for any input and handle it
        for event in events.poll_iter() {
            match event {
                // Break the event loop
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'event,
                // Handle other actual input
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(key) = keymap::keymap(key) {
                        chip8.key_press(key)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key_code),
                    ..
                } => {
                    if let Some(key) = keymap::keymap(key_code) {
                        chip8.key_release(key)
                    }
                }
                _ => continue,
            };
        }

        // Cycle the CPU
        chip8.cycle();

        // Handle Timing
        let current_time = Instant::now();
        let current_cycle_time = current_time - last_frame;
        if frame_rate > current_cycle_time {
            std::thread::sleep(frame_rate - current_cycle_time);
        }
        last_frame = Instant::now();
    }
}
