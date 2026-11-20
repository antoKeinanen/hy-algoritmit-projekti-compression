//! Types and helpers shared between the compression algorithm crates.

use bitvec::prelude::*;

/// Borrowed slice of bits, most significant bit first
pub type Bits = BitSlice<u8, Msb0>;

/// Owned, growable vector of bits, most significant bit first
pub type BitBuf = BitVec<u8, Msb0>;

/// A generic trait to compress a slice of bits
pub trait Compress {
    fn compress(&self, input: &Bits) -> BitBuf;
}

/// A generic trait to decompress a slice of bits
pub trait Decompress {
    fn decompress(&self, input: &Bits) -> BitBuf;
}

/// Append the `width` lowest bits of `value` to `output`, most significant bit first
pub fn write_bits(output: &mut BitBuf, value: usize, width: usize) {
    assert!(width <= usize::BITS as usize, "Bit width too large");
    for i in (0..width).rev() {
        output.push((value >> i) & 1 == 1);
    }
}

/// Read `width` bits from `input` starting at `pos` as an unsigned integer,
/// most significant bit first
pub fn read_bits(input: &Bits, pos: usize, width: usize) -> usize {
    assert!(width <= usize::BITS as usize, "Bit width too large");
    input[pos..pos + width]
        .iter()
        .by_vals()
        .fold(0usize, |acc, bit| (acc << 1) | bit as usize)
}

#[test]
fn test_bits_roundtrip() {
    let mut buf = BitBuf::new();
    write_bits(&mut buf, 0b101, 3);
    write_bits(&mut buf, 0xABCD, 16);
    write_bits(&mut buf, 0, 0);

    assert_eq!(buf.len(), 19);
    assert_eq!(read_bits(&buf, 0, 3), 0b101);
    assert_eq!(read_bits(&buf, 3, 16), 0xABCD);
    assert_eq!(read_bits(&buf, 3, 0), 0);
}

#[test]
fn test_write_bits_matches_view_bits() {
    let mut buf = BitBuf::new();
    write_bits(&mut buf, 1234, 16);
    assert_eq!(buf.as_bitslice(), 1234u16.view_bits::<Msb0>());
}
