use bitvec::prelude::*;

trait Compress {
    fn compress(&self, input: &BitSlice<u8, Msb0>) -> BitVec<u8, Msb0>;
}

trait Decompress {
    fn decompress(&self, input: &BitSlice<u8, Msb0>) -> BitVec<u8, Msb0>;
}

struct LZ77 {
    window_size: usize,
}

impl LZ77 {
    fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl Compress for LZ77 {
    fn compress(&self, input: &BitSlice<u8, Msb0>) -> BitVec<u8, Msb0> {
        let mut output = BitVec::<u8, Msb0>::new();
        let mut pos = 0;

        while pos < input.len() {
            let window_start = pos.saturating_sub(self.window_size);
            let mut best_offset = 0;
            let mut best_length = 0;

            for i in window_start..pos {
                let mut length = 0;

                while pos + length < input.len()
                    && length < self.window_size
                    && length < u16::MAX as usize
                    && input[i + length] == input[pos + length]
                {
                    length += 1;
                }

                if length > best_length {
                    best_length = length;
                    best_offset = pos - i;
                }
            }

            if pos + best_length >= input.len() {
                best_length = input.len() - pos - 1;
            }

            let offset = best_offset as u16;
            output.extend_from_bitslice(offset.view_bits::<Msb0>());

            let length = best_length as u16;
            output.extend_from_bitslice(length.view_bits::<Msb0>());

            let next = input[pos + best_length];
            output.push(next);

            pos += best_length + 1;
        }

        output
    }
}

impl Decompress for LZ77 {
    fn decompress(&self, input: &BitSlice<u8, Msb0>) -> BitVec<u8, Msb0> {
        let mut output = BitVec::<u8, Msb0>::new();
        let mut pos = 0;

        const TOKEN_SIZE: usize = 33;

        assert!(input.len() % TOKEN_SIZE == 0, "Invalid compressed data");

        while pos < input.len() {
            let offset = input[pos..pos + 16]
                .iter()
                .by_vals()
                .fold(0usize, |acc, bit| (acc << 1) | bit as usize);
            pos += 16;

            let length = input[pos..pos + 16]
                .iter()
                .by_vals()
                .fold(0usize, |acc, bit| (acc << 1) | bit as usize);
            pos += 16;

            let next = input[pos];
            pos += 1;

            if length > 0 {
                assert!(offset > 0 && offset <= output.len(), "Invalid LZ77 offset");

                for _ in 0..length {
                    let bit = output[output.len() - offset];
                    output.push(bit);
                }
            }

            output.push(next);
        }

        output
    }
}

#[test]
fn test_lz77_roundtrip() {
    let lz77 = LZ77::new(1024);

    let input = bitvec![u8, Msb0;
        1, 0, 1, 0, 1, 0, 1, 0,
        1, 0, 1, 0, 1, 0, 1, 0,
        1, 1, 1, 1, 0, 0, 0, 0
    ];

    let compressed = lz77.compress(&input);
    let decompressed = lz77.decompress(&compressed);

    assert_eq!(input, decompressed);
}
