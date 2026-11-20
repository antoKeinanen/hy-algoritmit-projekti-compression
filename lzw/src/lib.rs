use std::collections::HashMap;

use shared::{BitBuf, Bits, read_bits, write_bits};

pub use shared::{Compress, Decompress};

/// The alphabet consists of single bits: code 0 is `0` and code 1 is `1`
const ALPHABET_SIZE: usize = 2;

pub struct LZW {
    /// Maximum width of an emitted code in bits. The dictionary holds at most
    /// `2^max_code_bits` entries, after which no new entries are added.
    pub max_code_bits: usize,
}

impl LZW {
    /// Constructor for the LZW algorithm implementation
    pub fn new(max_code_bits: usize) -> Self {
        assert!(
            (1..=31).contains(&max_code_bits),
            "max_code_bits must be between 1 and 31"
        );
        Self { max_code_bits }
    }

    fn max_dict_size(&self) -> usize {
        1 << self.max_code_bits
    }
}

/// Number of bits needed to write any code of a dictionary with `size` entries
fn code_width(size: usize) -> usize {
    (usize::BITS - (size - 1).leading_zeros()).max(1) as usize
}

/// Implement the LZW compression algorithm
///
/// Codes are written with a variable width: each code uses just enough bits
/// to address every entry currently in the dictionary.
impl Compress for LZW {
    fn compress(&self, input: &Bits) -> BitBuf {
        let mut output = BitBuf::new();

        if input.is_empty() {
            return output;
        }

        let max_size = self.max_dict_size();
        // Dictionary as a trie: (prefix code, next bit) -> code
        let mut dict: HashMap<(usize, bool), usize> = HashMap::new();
        let mut dict_size = ALPHABET_SIZE;

        let mut current = input[0] as usize;

        for bit in input[1..].iter().by_vals() {
            if let Some(&code) = dict.get(&(current, bit)) {
                current = code;
                continue;
            }

            write_bits(&mut output, current, code_width(dict_size));

            if dict_size < max_size {
                dict.insert((current, bit), dict_size);
                dict_size += 1;
            }

            current = bit as usize;
        }

        write_bits(&mut output, current, code_width(dict_size));

        output
    }
}

/// Dictionary entries for decompression stored as prefix chains
struct DecodeDict {
    /// (prefix code, last bit) for each code; alphabet entries have no prefix
    entries: Vec<(Option<usize>, bool)>,
}

impl DecodeDict {
    fn new() -> Self {
        Self {
            entries: vec![(None, false), (None, true)],
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    /// Append the bit string of `code` to `output` and return its first bit
    fn write(&self, code: usize, output: &mut BitBuf) -> bool {
        let start = output.len();
        let mut next = Some(code);

        while let Some(code) = next {
            let (prefix, bit) = self.entries[code];
            output.push(bit);
            next = prefix;
        }

        output[start..].reverse();
        output[start]
    }
}

/// Implement the LZW decompression algorithm
impl Decompress for LZW {
    fn decompress(&self, input: &Bits) -> BitBuf {
        let mut output = BitBuf::new();

        if input.is_empty() {
            return output;
        }

        let max_size = self.max_dict_size();
        let mut dict = DecodeDict::new();
        let mut pos = 0;

        let width = code_width(dict.len());
        assert!(pos + width <= input.len(), "Invalid compressed data");
        let mut prev = read_bits(input, pos, width);
        pos += width;
        assert!(prev < dict.len(), "Invalid LZW code");
        dict.write(prev, &mut output);

        while pos < input.len() {
            // The decoder is one entry behind the encoder, as the entry is only
            // known once the first bit of the next string has been read
            let width = code_width((dict.len() + 1).min(max_size));
            assert!(pos + width <= input.len(), "Invalid compressed data");
            let code = read_bits(input, pos, width);
            pos += width;

            let full = dict.len() >= max_size;

            let first = if code < dict.len() {
                dict.write(code, &mut output)
            } else if code == dict.len() && !full {
                // Special case: the code refers to the entry being created,
                // which is the previous string followed by its own first bit
                let first = dict.write(prev, &mut output);
                output.push(first);
                first
            } else {
                panic!("Invalid LZW code");
            };

            if !full {
                dict.entries.push((Some(prev), first));
            }

            prev = code;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    fn roundtrip(lzw: &LZW, input: &Bits) {
        let compressed = lzw.compress(input);
        let decompressed = lzw.decompress(&compressed);
        assert_eq!(input, decompressed.as_bitslice());
    }

    #[test]
    fn test_lzw_roundtrip() {
        let lzw = LZW::new(12);

        let input = bitvec![u8, Msb0;
            1, 0, 1, 0, 1, 0, 1, 0,
            1, 0, 1, 0, 1, 0, 1, 0,
            1, 1, 1, 1, 0, 0, 0, 0
        ];

        roundtrip(&lzw, &input);
    }

    #[test]
    fn test_lzw_empty() {
        let lzw = LZW::new(12);
        let input = BitBuf::new();

        assert!(lzw.compress(&input).is_empty());
        roundtrip(&lzw, &input);
    }

    #[test]
    fn test_lzw_single_bit() {
        let lzw = LZW::new(12);
        roundtrip(&lzw, bits![u8, Msb0; 0]);
        roundtrip(&lzw, bits![u8, Msb0; 1]);
    }

    #[test]
    fn test_lzw_repeated_bits() {
        // Exercises the special case where a code refers to the entry being created
        let lzw = LZW::new(12);
        let input = bitvec![u8, Msb0; 1; 1000];
        roundtrip(&lzw, &input);
    }

    #[test]
    fn test_lzw_full_dictionary() {
        // A tiny dictionary fills up quickly, after which codes stay fixed width
        let lzw = LZW::new(3);
        let input = BitBuf::from_vec(b"hello hello hello world, the full dictionary".to_vec());
        roundtrip(&lzw, &input);
    }

    #[test]
    fn test_lzw_compresses_repetitive_input() {
        let lzw = LZW::new(16);
        let input = BitBuf::from_vec(b"abcabcabc".repeat(100));
        let compressed = lzw.compress(&input);

        assert!(compressed.len() < input.len());
        assert_eq!(lzw.decompress(&compressed), input);
    }

    #[test]
    fn test_code_width() {
        assert_eq!(code_width(1), 1);
        assert_eq!(code_width(2), 1);
        assert_eq!(code_width(3), 2);
        assert_eq!(code_width(4), 2);
        assert_eq!(code_width(5), 3);
        assert_eq!(code_width(1 << 12), 12);
    }

    #[test]
    #[should_panic(expected = "Invalid LZW code")]
    fn test_lzw_invalid_code() {
        let lzw = LZW::new(12);
        // First code 0 (1 bit), then a 2 bit code 3 when only code 2 can be next
        lzw.decompress(bits![u8, Msb0; 0, 1, 1]);
    }

    #[test]
    #[should_panic(expected = "Invalid compressed data")]
    fn test_lzw_truncated_input() {
        let lzw = LZW::new(12);
        // First code 0 (1 bit), then only 1 of the 2 bits needed for the next code
        lzw.decompress(bits![u8, Msb0; 0, 1]);
    }
}
