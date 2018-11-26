extern crate rand;
extern crate sdl2;

use display::Display;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

mod chip8;
mod display;
mod keymap;
mod opcode;
mod sprites;

fn main() {
    // TODO error handling
    // Initializing State
    let sdl = sdl2::init().unwrap();
    let mut chip8 = chip8::Chip8::new();
    let mut display = display::Window::new(&sdl, 64, 32, 10);
    let mut events = sdl.event_pump().unwrap();

    let args = std::env::args();
    if args.len() > 1 {
        let file_path = args.last().expect("unable to get file path from args");
        let file = File::open(file_path).expect("unable to open file");
        let mut reader = BufReader::new(file);
        chip8.load_rom(&mut reader);
    } else {
        panic!("expected ROM file path but got no arguments");
    }

    // Timing
    // the Chip-8 is generally emulated with a clock rate of 500Hz -> 2ms/cycle
    let cycle_time = Duration::new(0, 2_000_000);
    // the Chip-8 delay timers decrement at a constant rate of 60Hz -> ~16.7ms/tick
    let timer_time = Duration::new(0, 16_666_667);
    // sometimes it's useful to just skip through things
    let mut fast_forward = false;
    // set the initial timing for the timer/cycle timers
    let mut last_cycle = Instant::now();
    let mut last_timer = Instant::now();
    'event: loop {
        if chip8.should_draw {
            // Get the state of the Chip-8 FrameBuffer and draw it
            display.render(&chip8.frame_buffer);
        }

        // Check and handle input
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'event,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match (key, keymap::keymap(key)) {
                    (_, Some(kc)) => chip8.key_press(kc),
                    (Keycode::Space, _) => fast_forward = true,
                    _ => continue,
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match (key, keymap::keymap(key)) {
                    (_, Some(kc)) => chip8.key_release(kc),
                    (Keycode::Space, _) => fast_forward = false,
                    _ => continue,
                },
                _ => continue,
            };
        }

        // TODO rework this, remove repetition and address only decrementing during CPU cycles
        // Handle timer timing
        let current_time = Instant::now();
        let elapsed_timer_time = current_time - last_timer;
        if elapsed_timer_time > timer_time {
            chip8.should_decrement_timers = true;
            last_timer = Instant::now();
        }

        // Cycle the CPU
        chip8.cycle();

        // Handle cycle timing
        let current_time = Instant::now();
        let elapsed_cycle_time = current_time - last_cycle;
        if !fast_forward && cycle_time > elapsed_cycle_time {
            std::thread::sleep(cycle_time - elapsed_cycle_time);
        }
        last_cycle = Instant::now();
    }
}
