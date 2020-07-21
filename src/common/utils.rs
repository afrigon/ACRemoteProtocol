use bit_vec::BitVec;

pub trait AsBitVec {
    fn as_bitvec(self) -> BitVec;
}

pub fn bitreverse(x: &u8) -> u8 {
    (0..8_u8).fold(0, |acc, i| acc | (x >> i & 1) << (7 - i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_bitreversed_then_bits_are_reversed() {
        let data = 0b10010111;
        let expected = 0b11101001;

        assert_eq!(bitreverse(&data), expected);
    }
}
