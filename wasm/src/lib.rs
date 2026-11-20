//! WebAssembly bindings for the compression algorithms.
//!
//! Compressed data is a stream of bits whose length is not necessarily a
//! multiple of 8, so it is exposed to JavaScript as [`BitData`], which carries
//! the exact bit length alongside the bytes.

use shared::{BitBuf, Compress, Decompress};
use wasm_bindgen::prelude::*;

/// A sequence of bits stored as bytes, most significant bit first
#[wasm_bindgen]
pub struct BitData {
    bits: BitBuf,
}

#[wasm_bindgen]
impl BitData {
    /// Create bit data from `bytes`, keeping only the first `bit_len` bits
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>, bit_len: usize) -> Result<BitData, JsError> {
        if bit_len > bytes.len() * 8 {
            return Err(JsError::new("bit_len exceeds the number of bits in bytes"));
        }
        let mut bits = BitBuf::from_vec(bytes);
        bits.truncate(bit_len);
        Ok(Self { bits })
    }

    /// The bits packed into bytes, with the last byte padded with zeros
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        to_bytes(&self.bits)
    }

    /// The exact number of bits
    #[wasm_bindgen(getter, js_name = bitLen)]
    pub fn bit_len(&self) -> usize {
        self.bits.len()
    }
}

fn to_bytes(bits: &BitBuf) -> Vec<u8> {
    let mut bits = bits.clone();
    bits.set_uninitialized(false);
    bits.into_vec()
}

fn compress(algorithm: &impl Compress, input: Vec<u8>) -> BitData {
    BitData {
        bits: algorithm.compress(&BitBuf::from_vec(input)),
    }
}

fn decompress(algorithm: &impl Decompress, input: &BitData) -> Vec<u8> {
    to_bytes(&algorithm.decompress(&input.bits))
}

/// LZ77 compression
#[wasm_bindgen]
pub struct Lz77 {
    inner: lz77::LZ77,
}

#[wasm_bindgen]
impl Lz77 {
    #[wasm_bindgen(constructor)]
    pub fn new(window_size: usize) -> Lz77 {
        Self {
            inner: lz77::LZ77::new(window_size),
        }
    }

    pub fn compress(&self, input: Vec<u8>) -> BitData {
        compress(&self.inner, input)
    }

    pub fn decompress(&self, input: &BitData) -> Vec<u8> {
        decompress(&self.inner, input)
    }
}

/// LZW compression
#[wasm_bindgen]
pub struct Lzw {
    inner: lzw::LZW,
}

#[wasm_bindgen]
impl Lzw {
    #[wasm_bindgen(constructor)]
    pub fn new(max_code_bits: usize) -> Result<Lzw, JsError> {
        if !(1..=31).contains(&max_code_bits) {
            return Err(JsError::new("max_code_bits must be between 1 and 31"));
        }
        Ok(Self {
            inner: lzw::LZW::new(max_code_bits),
        })
    }

    pub fn compress(&self, input: Vec<u8>) -> BitData {
        compress(&self.inner, input)
    }

    pub fn decompress(&self, input: &BitData) -> Vec<u8> {
        decompress(&self.inner, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &[u8] = b"abracadabra abracadabra abracadabra";

    #[test]
    fn test_lz77_roundtrip() {
        let lz77 = Lz77::new(64);
        let compressed = lz77.compress(INPUT.to_vec());
        assert_eq!(lz77.decompress(&compressed), INPUT);
    }

    #[test]
    fn test_lzw_roundtrip() {
        let lzw = Lzw::new(12).unwrap();
        let compressed = lzw.compress(INPUT.to_vec());
        assert_eq!(lzw.decompress(&compressed), INPUT);
    }

    #[test]
    fn test_bit_data_roundtrip_through_bytes() {
        // Simulates passing compressed data to JavaScript and back
        let lzw = Lzw::new(12).unwrap();
        let compressed = lzw.compress(INPUT.to_vec());
        let restored = BitData::new(compressed.bytes(), compressed.bit_len()).unwrap();

        assert_eq!(restored.bit_len(), compressed.bit_len());
        assert_eq!(lzw.decompress(&restored), INPUT);
    }
}
