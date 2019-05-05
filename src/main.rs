extern crate rand;
extern crate sdl2;

use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod chip8;
mod constants;
mod display;
mod keymap;
mod opcode;
mod state;

// TODO error handling
fn main() {
    let mut chip8 = chip8::Chip8::new();

    // Get SDL2 context
    let sdl = sdl2::init().unwrap();
    let mut display = display::Display::new(
        &sdl,
        constants::DISPLAY_WIDTH,
        constants::DISPLAY_HEIGHT,
        10,
    );
    let mut events = sdl.event_pump().unwrap();

    // Load ROM
    let args = std::env::args();
    if args.len() > 1 {
        let file_path = args.last().expect("unable to get file path from args");
        let file = File::open(file_path).expect("unable to open file");
        let mut reader = BufReader::new(file);
        chip8.load_rom(&mut reader);
    } else {
        panic!("expected ROM file path but got no arguments");
    }

    // Set initial timing
    let cycle_time = Duration::new(0, constants::CLOCK_SPEED as u32);
    let mut last_cycle = Instant::now();
    let mut fast_forward = false;

    // Whether the game should be run forwards or in reverse
    let mut rewind = false;

    // TODO handle saving/reloading/rewinding state
    'event: loop {
        // If the draw flag is set, unset it and render the current frame
        if let Some(frame) = chip8.get_frame() {
            display.render(&frame);
        }

        // Handle input
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'event,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match (key, keymap::keymap(key)) {
                    (_, Some(kc)) => chip8.key_press(kc),
                    (Keycode::Space, _) => fast_forward = true,
                    (Keycode::Escape, _) => rewind = true,
                    _ => continue,
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match (key, keymap::keymap(key)) {
                    (_, Some(kc)) => chip8.key_release(kc),
                    (Keycode::Space, _) => fast_forward = false,
                    (Keycode::Escape, _) => rewind = false,
                    _ => continue,
                },
                _ => continue,
            };
        }

        // Update state
        if rewind {
            chip8.reverse_cycle();
        } else {
            chip8.advance_cycle();
            chip8.cycle_timers();
        }

        // Handle timing
        let current_time = Instant::now();
        let elapsed_cycle_time = current_time - last_cycle;
        if !fast_forward && cycle_time > elapsed_cycle_time {
            std::thread::sleep(cycle_time - elapsed_cycle_time);
        }
        last_cycle = current_time;
    }
}
