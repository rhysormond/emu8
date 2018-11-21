use sdl2::keyboard::Keycode;

/// # Keymap
/// Chip-8 input is generated with a hexadecimal keypad
pub fn keymap(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Num0 => Some(0x0),
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0x4),
        Keycode::Num5 => Some(0x5),
        Keycode::Num6 => Some(0x6),
        Keycode::Num7 => Some(0x7),
        Keycode::Num8 => Some(0x8),
        Keycode::Num9 => Some(0x9),
        Keycode::A => Some(0xA),
        Keycode::B => Some(0xB),
        Keycode::C => Some(0xC),
        Keycode::D => Some(0xD),
        Keycode::E => Some(0xE),
        Keycode::F => Some(0xF),
        _ => None,
    }
}
