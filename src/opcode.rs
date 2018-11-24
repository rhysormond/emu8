/// # Opcodes
/// Chip-8 opcodes are 16 bit. Opcodes' general behavior is determined
/// by the opcode's nibbles, generally the first and last ones.
pub trait Opcode
where
    Self: Sized,
{
    /// Returns the Opcode decomposed as an array of 4 nibbles.
    /// (represented as u8 since there's no u4)
    fn as_nibbles(&self) -> (u8, u8, u8, u8);
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
}

#[cfg(test)]
mod test_opcode {
    use super::*;

    #[test]
    fn as_nibbles() {
        let op: u16 = 0xABCD;
        assert_eq!(op.as_nibbles(), (0xA, 0xB, 0xC, 0xD));
    }
}
