use bitvec::prelude::*;
use shared::{Bits, Traced, read_bits};

pub use shared::{Compress, Decompress, Span};

/// Size of one encoded token: 16 bit offset, 16 bit length and 1 literal bit
const TOKEN_SIZE: usize = 33;

pub struct LZ77 {
    pub window_size: usize,
}

impl LZ77 {
    /// Constructor for the LZ77 algorithm implementation
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

/// Implement the LZ77 compression algorithm
///
/// Each span links the input bits covered by one token to the encoded token.
impl Compress for LZ77 {
    fn compress_traced(&self, input: &Bits) -> Traced {
        let mut traced = Traced::new();
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

            let output_start = traced.output.len();

            let offset = best_offset as u16;
            traced
                .output
                .extend_from_bitslice(offset.view_bits::<Msb0>());

            let length = best_length as u16;
            traced
                .output
                .extend_from_bitslice(length.view_bits::<Msb0>());

            let next = input[pos + best_length];
            traced.output.push(next);

            let token_end = pos + best_length + 1;
            traced.record(pos..token_end, output_start);
            pos = token_end;
        }

        traced
    }
}

/// Implement the LZ77 decompression algorithm
///
/// Each span links one encoded token to the bits it decoded into.
impl Decompress for LZ77 {
    fn decompress_traced(&self, input: &Bits) -> Traced {
        let mut traced = Traced::new();
        let mut pos = 0;

        assert!(input.len().is_multiple_of(TOKEN_SIZE), "Invalid compressed data");

        while pos < input.len() {
            let token_start = pos;
            let output_start = traced.output.len();

            let offset = read_bits(input, pos, 16);
            pos += 16;

            let length = read_bits(input, pos, 16);
            pos += 16;

            let next = input[pos];
            pos += 1;

            let output = &mut traced.output;
            if length > 0 {
                assert!(offset > 0 && offset <= output.len(), "Invalid LZ77 offset");

                for _ in 0..length {
                    let bit = output[output.len() - offset];
                    output.push(bit);
                }
            }

            output.push(next);

            traced.record(token_start..pos, output_start);
        }

        traced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_lz77_spans() {
        let lz77 = LZ77::new(1024);
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 1];

        // Tokens: literal 1, literal 0, copy "10101" + literal 1
        let compressed = lz77.compress_traced(&input);
        assert!(compressed.spans_cover(input.len()));
        assert_eq!(
            compressed.spans,
            vec![
                Span {
                    input: 0..1,
                    output: 0..33
                },
                Span {
                    input: 1..2,
                    output: 33..66
                },
                Span {
                    input: 2..8,
                    output: 66..99
                },
            ]
        );

        // Decompression spans mirror the compression spans
        let decompressed = lz77.decompress_traced(&compressed.output);
        assert_eq!(decompressed.output, input);
        assert!(decompressed.spans_cover(compressed.output.len()));
        for (c, d) in compressed.spans.iter().zip(&decompressed.spans) {
            assert_eq!(c.input, d.output);
            assert_eq!(c.output, d.input);
        }
    }

    #[test]
    fn test_lz77_empty_spans() {
        let lz77 = LZ77::new(1024);
        let traced = lz77.compress_traced(&BitVec::<u8, Msb0>::new());
        assert!(traced.output.is_empty());
        assert!(traced.spans.is_empty());
    }
}
