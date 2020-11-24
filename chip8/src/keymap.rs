use sdl2::keyboard::Keycode;

/// # Keymap
/// Chip-8 input is generated with a hexadecimal keypad.
///
/// This original layout is mapped to the left 4 alphanumeric columns.
/// ```text
/// |1|2|3|C|      |1|2|3|4|
/// |4|5|6|D|  ->  |Q|W|E|R|
/// |7|8|9|E|  ->  |A|S|D|F|
/// |A|0|B|F|      |Z|X|C|V|
/// ```
pub fn keymap(key: Keycode) -> Option<u8> {
    match key {
        Keycode::X => Some(0x0),
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::Z => Some(0xA),
        Keycode::C => Some(0xB),
        Keycode::Num4 => Some(0xC),
        Keycode::R => Some(0xD),
        Keycode::F => Some(0xE),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
