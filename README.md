# MNIST

The ONNX Model Zoo's int8 MNIST network, authored as one LexLean model graph and
compiled to a certified `rust-std` root. The graph under `src/Mnist/` is the
single source: its Lean, its LaTeX, its kernel theorems, its production
certificates and its Rust rendering are all derived from it by the locked
LexLean compiler, and nothing in this repository implements the model in any
other language (R1).

## The model

`mnist-12-int8.onnx` (CNTK-trained, quantized with Intel Neural Compressor;
see `model/authorities.toml`): uint8 activations with zero point 0, int8
weights with zero point 0, int32 biases, about 6,000 parameters.

| ONNX graph | LexLean realization (module) |
| --- | --- |
| QuantizeLinear on raw 0–255 pixels | identity; the image enters as 28 rows of 28 `Int` (`Network`) |
| QLinearConv 5×5 SAME 1→8 + bias | one fused fold per output row over five padded rows and the accumulator row (`Conv.fusedRow`, `Conv.convAcc`) |
| MaxPool 2×2 | folds pairing adjacent rows and values (`Pool.pool2`) |
| QLinearConv 5×5 SAME 8→16 + bias | the same pass accumulated over the eight input planes (`Network.conv2Channel*`) |
| Dequantize, MaxPool 3×3, Reshape, Quantize | integer 3×3 pool on the codes and a flatten in channel, row, column order (`Pool.pool3`, `Network.flatten`) |
| QLinearMatMul 256×10, QLinearAdd, Dequantize | ten dot products and two fixed-point requantizations, then argmax (`Network.outputCode`, `Core.argmax`) |

Requantization is exact integer arithmetic,
`clamp((acc + bias) · M + 2³¹ + Z · 2³² quot 2³², 0, 255)` with
`M = round(s_in · s_w / s_out · 2³²)` per channel; the multipliers are the
literals in `Network` and `Params`. Every input-sized loop is a `list_fold`;
there is no indexing and no recursion, and the root `Network.classify` admits
exactly the `allocation` and `overflow` effects.

## What is established, and what is measured

`lexlean verify` on the committed graph replays 17 kernel theorems on a real
test image (a fused row, a requantized row, one complete conv1 channel, both
pools, the flatten order, all ten output codes and the argmax), audits every
declaration's exact axiom policy, and emits the three preservation certificates
of `classify`: source to calculus, calculus to the declared Rust machine, and
their composition. The record of the verification this repository ships is
`docs/evidence/verification.json`, bound to the committed source by `MN-03`.

Whole-image classification is not a kernel theorem (a forward pass exceeds the
kernel child limit); it is measured, with the certified rendering, against
onnxruntime over the full MNIST test set. Those measurements are `open` claims
in `model/ledger.toml` and are reported in `docs/evidence/head-to-head.md`:
the same class as onnxruntime's int8 run on all 10,000 images, 98.92%
accuracy, and 28.7 ms per image against 66 µs, with an identical work count on
every image.

## Layout

| Path | Role |
| --- | --- |
| `src/Mnist/Core.lex.tex` | list utilities, requantization, dot product, argmax |
| `src/Mnist/Conv.lex.tex` | the fused 5×5 convolution pass |
| `src/Mnist/Pool.lex.tex` | 2×2 and 3×3 max pooling |
| `src/Mnist/Params.lex.tex` | fully connected weights and the output quantization constants |
| `src/Mnist/Network.lex.tex` | the layers, the convolution kernels, and the `classify` root |
| `src/Mnist/Scenarios.lex.tex` | the kernel-checked theorems |
| `lexlean.toml`, `lexlean.lock` | the LexLean project (language 1.2) and its lock |
| `prismpm.toml` | the PrismPM project |
| `model/`, `features/suites/` | the conformance register, authorities, ledger and scenarios |
| `docs/evidence/` | the verification record and the measurements |

## Gates

`just vv` is the acceptance gate. `just model-verify` runs the LexLean side
alone: `lock --check`, `fmt --check --all`, `check --all`, `build`, `verify`.
The conformance tests `MN-01`..`MN-03` drive the same `lexlean` command from
`cargo test` and read what it writes.

This repository is instantiated from
[UOR-Foundation/template](https://github.com/UOR-Foundation/template) at
`1bea460bac6ea50bae53a7eeb674589d7900e6cb`. The template is not yet bound to a
public PrismPM SDK release, so `prismpm.lock` and `template.lock` are absent
and the SDK gates (`prismpm template check`, `prismpm lock check`) wait on that
binding, as in the template itself.

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
