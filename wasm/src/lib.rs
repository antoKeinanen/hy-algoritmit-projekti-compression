//! WebAssembly bindings for the compression algorithms.
//!
//! Compressed data is a stream of bits whose length is not necessarily a
//! multiple of 8, so it is exposed to JavaScript as [`BitData`], which carries
//! the exact bit length alongside the bytes.
//!
//! The `*Traced` methods also return [`BitSpan`]s that link ranges of input
//! bits to the ranges of output bits they produced.

use shared::{BitBuf, Compress, Decompress, Traced};
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

/// Input bits `inputStart..inputEnd` produced output bits `outputStart..outputEnd`
/// (end exclusive)
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitSpan {
    #[wasm_bindgen(js_name = inputStart)]
    pub input_start: usize,
    #[wasm_bindgen(js_name = inputEnd)]
    pub input_end: usize,
    #[wasm_bindgen(js_name = outputStart)]
    pub output_start: usize,
    #[wasm_bindgen(js_name = outputEnd)]
    pub output_end: usize,
}

/// Output of an algorithm together with the spans that produced it
#[wasm_bindgen]
pub struct TracedData {
    output: BitBuf,
    spans: Vec<BitSpan>,
}

#[wasm_bindgen]
impl TracedData {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> BitData {
        BitData {
            bits: self.output.clone(),
        }
    }

    /// Spans in order, together covering the whole input and output
    #[wasm_bindgen(getter)]
    pub fn spans(&self) -> Vec<BitSpan> {
        self.spans.clone()
    }
}

impl From<Traced> for TracedData {
    fn from(traced: Traced) -> Self {
        Self {
            output: traced.output,
            spans: traced
                .spans
                .into_iter()
                .map(|span| BitSpan {
                    input_start: span.input.start,
                    input_end: span.input.end,
                    output_start: span.output.start,
                    output_end: span.output.end,
                })
                .collect(),
        }
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

fn compress_traced(algorithm: &impl Compress, input: Vec<u8>) -> TracedData {
    algorithm.compress_traced(&BitBuf::from_vec(input)).into()
}

fn decompress_traced(algorithm: &impl Decompress, input: &BitData) -> TracedData {
    algorithm.decompress_traced(&input.bits).into()
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

    #[wasm_bindgen(js_name = compressTraced)]
    pub fn compress_traced(&self, input: Vec<u8>) -> TracedData {
        compress_traced(&self.inner, input)
    }

    #[wasm_bindgen(js_name = decompressTraced)]
    pub fn decompress_traced(&self, input: &BitData) -> TracedData {
        decompress_traced(&self.inner, input)
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

    #[wasm_bindgen(js_name = compressTraced)]
    pub fn compress_traced(&self, input: Vec<u8>) -> TracedData {
        compress_traced(&self.inner, input)
    }

    #[wasm_bindgen(js_name = decompressTraced)]
    pub fn decompress_traced(&self, input: &BitData) -> TracedData {
        decompress_traced(&self.inner, input)
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

    #[test]
    fn test_traced_spans() {
        let lzw = Lzw::new(12).unwrap();
        let compressed = lzw.compress_traced(INPUT.to_vec());
        let compressed_data = compressed.output();
        let decompressed = lzw.decompress_traced(&compressed_data);

        assert_eq!(decompressed.output().bytes(), INPUT);
        assert_eq!(
            compressed.spans().last().unwrap().output_end,
            compressed_data.bit_len()
        );
        assert_eq!(compressed.spans().len(), decompressed.spans().len());
        for (c, d) in compressed.spans().iter().zip(decompressed.spans()) {
            assert_eq!((c.input_start, c.input_end), (d.output_start, d.output_end));
            assert_eq!((c.output_start, c.output_end), (d.input_start, d.input_end));
        }
    }
}
