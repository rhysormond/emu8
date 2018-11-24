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
