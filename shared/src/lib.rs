//! Types and helpers shared between the compression algorithm crates.

use std::ops::Range;

use bitvec::prelude::*;

/// Borrowed slice of bits, most significant bit first
pub type Bits = BitSlice<u8, Msb0>;

/// Owned, growable vector of bits, most significant bit first
pub type BitBuf = BitVec<u8, Msb0>;

/// Links a range of input bits to the range of output bits they produced.
///
/// For compression, `input` is the part of the original data consumed by one
/// token and `output` is the encoded token. For decompression it is the other
/// way around: `input` is the encoded token and `output` the decoded data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub input: Range<usize>,
    pub output: Range<usize>,
}

/// Output of an algorithm together with the spans that produced it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traced {
    pub output: BitBuf,
    /// Spans in order. Together they cover the whole input and output.
    pub spans: Vec<Span>,
}

impl Traced {
    pub fn new() -> Self {
        Self {
            output: BitBuf::new(),
            spans: Vec::new(),
        }
    }

    /// Record that input bits `input` produced all output written since `output_start`
    pub fn record(&mut self, input: Range<usize>, output_start: usize) {
        self.spans.push(Span {
            input,
            output: output_start..self.output.len(),
        });
    }

    /// Find the span that produced the output bit at `index`
    pub fn span_for_output(&self, index: usize) -> Option<&Span> {
        find_span(&self.spans, index, |span| &span.output)
    }

    /// Find the span that consumed the input bit at `index`
    pub fn span_for_input(&self, index: usize) -> Option<&Span> {
        find_span(&self.spans, index, |span| &span.input)
    }

    /// Check that the spans are in order and together cover exactly
    /// `input_len` input bits and all of the output, without gaps or overlaps
    pub fn spans_cover(&self, input_len: usize) -> bool {
        let mut input_end = 0;
        let mut output_end = 0;

        for span in &self.spans {
            if span.input.start != input_end
                || span.output.start != output_end
                || span.input.end < span.input.start
                || span.output.end < span.output.start
            {
                return false;
            }
            input_end = span.input.end;
            output_end = span.output.end;
        }

        input_end == input_len && output_end == self.output.len()
    }
}

impl Default for Traced {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary search the ordered `spans` for the one whose `range` contains `index`
fn find_span(
    spans: &[Span],
    index: usize,
    range: impl Fn(&Span) -> &Range<usize>,
) -> Option<&Span> {
    let i = spans.partition_point(|span| range(span).end <= index);
    spans.get(i).filter(|span| range(span).contains(&index))
}

/// A generic trait to compress a slice of bits
pub trait Compress {
    /// Compress `input`, recording which input bits produced each part of the output
    fn compress_traced(&self, input: &Bits) -> Traced;

    fn compress(&self, input: &Bits) -> BitBuf {
        self.compress_traced(input).output
    }
}

/// A generic trait to decompress a slice of bits
pub trait Decompress {
    /// Decompress `input`, recording which input bits produced each part of the output
    fn decompress_traced(&self, input: &Bits) -> Traced;

    fn decompress(&self, input: &Bits) -> BitBuf {
        self.decompress_traced(input).output
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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

    fn sample() -> Traced {
        let mut traced = Traced::new();
        write_bits(&mut traced.output, 0, 4);
        traced.record(0..2, 0);
        write_bits(&mut traced.output, 0, 3);
        traced.record(2..10, 4);
        traced
    }

    #[test]
    fn test_record_and_lookup() {
        let traced = sample();

        assert_eq!(
            traced.spans,
            vec![
                Span {
                    input: 0..2,
                    output: 0..4
                },
                Span {
                    input: 2..10,
                    output: 4..7
                },
            ]
        );
        assert_eq!(traced.span_for_output(3), Some(&traced.spans[0]));
        assert_eq!(traced.span_for_output(4), Some(&traced.spans[1]));
        assert_eq!(traced.span_for_output(7), None);
        assert_eq!(traced.span_for_input(9), Some(&traced.spans[1]));
        assert_eq!(traced.span_for_input(10), None);
    }

    #[test]
    fn test_spans_cover() {
        let traced = sample();
        assert!(traced.spans_cover(10));
        assert!(!traced.spans_cover(11));

        let mut gap = traced.clone();
        gap.spans[1].input.start = 3;
        assert!(!gap.spans_cover(10));

        assert!(Traced::new().spans_cover(0));
    }
}
