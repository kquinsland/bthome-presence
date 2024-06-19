/// Converts a single nibble (4 bits) to a hexadecimal character.
fn nibble_to_hex_char(nibble: u8) -> char {
    match nibble {
        0x0..=0x9 => (b'0' + nibble) as char,
        0xa..=0xf => (b'a' + nibble - 10) as char,
        _ => '?', // Should never happen if used correctly
    }
}

/// Converts a byte to a two-character hexadecimal string.
pub fn byte_to_hex(byte: u8) -> [char; 2] {
    let high = (byte >> 4) & 0x0F;
    let low = byte & 0x0F;
    [nibble_to_hex_char(high), nibble_to_hex_char(low)]
}

#[cfg(test)]
mod tests {
    // TODO: paths are a bit wonky, still need to fix them
    use crate::common::util::encoding::*;

    #[test]
    fn test_nibble_to_hex_char() {
        assert_eq!(nibble_to_hex_char(0x0), '0');
        assert_eq!(nibble_to_hex_char(0x1), '1');
        assert_eq!(nibble_to_hex_char(0x9), '9');
        assert_eq!(nibble_to_hex_char(0xa), 'a');
        assert_eq!(nibble_to_hex_char(0xf), 'f');
    }

    #[test]
    #[should_panic]
    fn test_nibble_to_hex_char_invalid() {
        nibble_to_hex_char(0x10); // This should panic as it's not a valid nibble
    }

    #[test]
    fn test_byte_to_hex() {
        assert_eq!(byte_to_hex(0x00), ['0', '0']);
        assert_eq!(byte_to_hex(0x01), ['0', '1']);
        assert_eq!(byte_to_hex(0x0f), ['0', 'f']);
        assert_eq!(byte_to_hex(0xff), ['f', 'f']);
        assert_eq!(byte_to_hex(0xab), ['a', 'b']);
    }
}
