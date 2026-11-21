import { useMemo, useState } from "react";
import { BitData, Lz77, Lzw, type TracedData } from "wasm";

type Algorithm = "lz77" | "lzw";

const DEFAULT_PARAM: Record<Algorithm, number> = { lz77: 256, lzw: 12 };
const PARAM_NAME: Record<Algorithm, string> = {
  lz77: "Window size",
  lzw: "Max code bits",
};

type Span = {
  inputStart: number;
  inputEnd: number;
  outputStart: number;
  outputEnd: number;
};

type Result = {
  inputBits: number;
  outputBits: number;
  spans: Span[];
  input: Uint8Array;
  output: Uint8Array;
};

/** Bits `start..end` of `bytes` as a string of 0s and 1s, most significant bit first */
function bitString(bytes: Uint8Array, start: number, end: number): string {
  let s = "";
  for (let i = start; i < end; i++) {
    s += (bytes[i >> 3] >> (7 - (i & 7))) & 1;
  }
  return s;
}

/** Copy the spans out of wasm memory and free the wasm objects */
function takeSpans(traced: TracedData): Span[] {
  return traced.spans.map((span) => {
    const plain = {
      inputStart: span.inputStart,
      inputEnd: span.inputEnd,
      outputStart: span.outputStart,
      outputEnd: span.outputEnd,
    };
    span.free();
    return plain;
  });
}

function run(algorithm: Algorithm, param: number, text: string): Result {
  const input = new TextEncoder().encode(text);
  const codec = algorithm === "lz77" ? new Lz77(param) : new Lzw(param);
  let compressed: TracedData | undefined;
  let compressedData: BitData | undefined;

  try {
    compressed = codec.compressTraced(input);
    compressedData = compressed.output;

    return {
      inputBits: input.length * 8,
      outputBits: compressedData.bitLen,
      spans: takeSpans(compressed),
      input,
      output: compressedData.bytes,
    };
  } finally {
    codec.free();
    compressed?.free();
    compressedData?.free();
  }
}

export default function App() {
  const [algorithm, setAlgorithm] = useState<Algorithm>("lz77");
  const [param, setParam] = useState(DEFAULT_PARAM.lz77);
  const [text, setText] = useState("abracadabra abracadabra");

  const result = useMemo(() => {
    try {
      return run(algorithm, param, text);
    } catch (error) {
      return String(error);
    }
  }, [algorithm, param, text]);

  return (
    <>
      <p>
        <label>
          Algorithm{" "}
          <select
            value={algorithm}
            onChange={(e) => {
              const next = e.target.value as Algorithm;
              setAlgorithm(next);
              setParam(DEFAULT_PARAM[next]);
            }}
          >
            <option value="lz77">LZ77</option>
            <option value="lzw">LZW</option>
          </select>
        </label>{" "}
        <label>
          {PARAM_NAME[algorithm]}{" "}
          <input
            type="number"
            value={param}
            onChange={(e) => setParam(Number(e.target.value))}
          />
        </label>
      </p>

      <p>
        <textarea
          rows={5}
          cols={60}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </p>

      {typeof result === "string" ? (
        <p>Error: {result}</p>
      ) : (
        <>
          <p>
            {result.inputBits} bits {"->"} {result.outputBits} bits
            {result.inputBits > 0 &&
              ` (${((result.outputBits / result.inputBits) * 100).toFixed(1)} %)`}
          </p>

          <table border={1}>
            <thead>
              <tr>
                <th>#</th>
                <th>Input range</th>
                <th>Input bits</th>
                <th>Output range</th>
                <th>Output bits</th>
              </tr>
            </thead>
            <tbody>
              {result.spans.map((span, i) => (
                <tr key={i}>
                  <td>{i}</td>
                  <td>
                    {span.inputStart}..{span.inputEnd}
                  </td>
                  <td>
                    <code>
                      {bitString(result.input, span.inputStart, span.inputEnd)}
                    </code>
                  </td>
                  <td>
                    {span.outputStart}..{span.outputEnd}
                  </td>
                  <td>
                    <code>
                      {bitString(result.output, span.outputStart, span.outputEnd)}
                    </code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </>
  );
}
