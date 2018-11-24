/// # Opcodes
///
/// Chip-8 opcodes are 16 bits each. Their behavior is cased on some combination of:
/// - (n, _, _, _) broad categorization; applies to all opcodes
/// - (_, _, _, n) specific behavior within a category
/// - (_, _, n, n) more specific behavior within a category
/// - (_, n, n, n) some fixed function that doesn't require variables (e.g. CLS; clear screen)
///
/// Nibbles not used to determine the operation often (but not always) carry important data.
/// - (_, n, n, n)
///     - the three nibbles together almost always represent a 16-bit address
///     - the exception is DRW which treats these as x, y, and height coordinates respectively
/// - (_, n, _, _) always refers either to the register Vx or a range of registers V0..Vx
/// - (_, _, n, _) always refers to the the register Vy
pub trait Opcode
where
    Self: Sized,
{
    /// Returns the Opcode decomposed as an array of 4 nibbles.
    fn as_nibbles(&self) -> (u8, u8, u8, u8);

    /// Returns the Opcode's three least significant nibbles.
    /// These are used together to refer to a memory address.
    fn addr(&self) -> u16;
}

impl Opcode for u16 {
    fn as_nibbles(&self) -> (u8, u8, u8, u8) {
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
}

#[cfg(test)]
mod test_opcode {
    use super::*;

    #[test]
    fn as_nibbles() {
        let op: u16 = 0xABCD;
        assert_eq!(op.as_nibbles(), (0xA, 0xB, 0xC, 0xD));
    }

    #[test]
    fn addr() {
        let op: u16 = 0xABCD;
        assert_eq!(op.addr(), 0x0BCD);
    }
}
