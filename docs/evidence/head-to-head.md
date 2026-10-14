# Head-to-head: the certified rendering against onnxruntime

Measurements, not claims (R2): the rows of `model/ledger.toml` tagged `open`
report these numbers and never assert them. Everything here was taken on one
2-vCPU Linux container, single-threaded, over the 10,000-image MNIST test set
(`MNIST-T10K` in `model/authorities.toml`).

## Method

- Baselines: onnxruntime 1.29.0, CPU execution provider, one intra-op thread,
  running `mnist-12.onnx` (float32) and `mnist-12-int8.onnx` (`ONNX-MNIST-12-FP32`,
  `ONNX-MNIST-12-INT8`). The int8 model's input has scale 1.0 and zero point 0, so
  it takes raw 0–255 pixel values; fed the README's [0, 1] floats it quantizes
  every pixel to 0 or 1 and reaches 54.5%, which is worth knowing before
  comparing anything against it.
- LexLean: the package `dist/packages/rust-std/mnist` the committed bundle
  carries (its crate the one `result.json` binds), compiled unchanged by
  `rustc` and called once per image with the 784 raw pixel bytes through its
  export `run`, which checks the precondition, converts the bytes to rows,
  runs the network and returns the digit. `MN-05` is this run as a gate; the
  timing driver is test text and not part of this repository.
- Agreement is measured on the predicted class; for the ten output codes, on
  the uint8 code onnxruntime's dequantized logit rounds back to.

## Results

| Run | Accuracy | Errors | Top-1 agreement with ORT int8 | Top-1 agreement with fp32 | Time per image |
| --- | --- | --- | --- | --- | --- |
| onnxruntime, mnist-12.onnx (fp32) | 98.90% | 110 | 99.65% | — | 61 µs |
| onnxruntime, mnist-12-int8.onnx | 98.92% | 108 | — | 99.65% | 66 µs |
| LexLean certified package (`Mnist.Model.classify`, bytes in) | 98.92% | 108 | 100.00% | 99.65% | 8.7 ms |

The zoo README reports 1.1% top-1 error for this model; fp32 reproduces it
exactly (110 errors). Of the 35 images where int8 and fp32 disagree, the int8
answer is right on 17 and fp32 on 15.

Output codes, not only classes: the rendering's ten uint8 codes are identical
to onnxruntime's on 99.95% of images; on the remaining five images exactly one
code differs by 1 and the class is unchanged. onnxruntime requantizes with a
float32 multiply and rounds the float; the LexLean graph uses the 32-bit
fixed-point multiplier `round(s_in · s_w / s_out · 2³²)`. The two agree except
when the exact product lies within float32 rounding of a half-integer. The
integer arithmetic is the one that is exactly specified; this is why agreement
with onnxruntime is reported as a rate rather than claimed as bit-exactness.
Bit-exactness holds between the graph, the kernel and the rendering, by
certificate.

## Time

The rendering's work counter reads 52,365 steps on all 10,000 images: the
calculus charges by the program's structure, not by the data. Wall time is
8.7 ms per image over the full run with the package LexLean certifies for the
model root (crate sha256 `44d2b35f…`).

Of that, 7.7 ms is the list-graph classifier `Network.classifyRows`, which the
same compiler certified alone at 39,944 steps before the model boundary
existed; the graph took 28.7 ms under the LexLean of a day earlier, and the
difference is the `rust-std` rendering, not the graph: LexLean #87 made a last
read a move, a last projection a move of the component, and a list a run of
contiguous cells (`rendering.md` has the measurements that led to it). The
remaining 1.0 ms is the product boundary and the parameters as data: decoding
784 pixel bytes into integers (the language has no byte-to-integer
conversion, so each goes through its decimal text), materializing the ten
parameter tensors from their artifact literals once per image, taking the
twenty-five taps off each of the 136 kernels, and transposing the 256×10
weight table once. What remains in the 7.7 ms is the representation: a fold
state is passed by value and its list views are reference-counted, so the
convolutions still pay per state read and per view; onnxruntime runs the same
arithmetic on contiguous int8 arrays with SIMD kernels. The same arithmetic
over contiguous `i64` arrays runs at 0.74 ms, which is the floor for this
graph's rendering (LexLean #84).

Two orderings of the same convolution were measured. The first assembled each
5×5×8 output row from 40 correlation passes and 47 elementwise additions and
took 51.9 ms per image under the old rendering. The committed graph walks the
five rows and the running accumulator in one fold per output row and
destructures its state tree once per level; that is the 28.7 ms then and the
7.7 ms now. The predictions of the two are identical on all 10,000 images.
