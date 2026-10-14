# MNIST

The ONNX Model Zoo's int8 MNIST network as one LexLean model: its parameters
are content-addressed artifacts, its network is a language-1.2 graph, its
boundary is a contract with validators and evidence, and its product is a
certified Rust package bundle that replays offline. Everything under
`src/Mnist/` is the single source; the Lean, the LaTeX, the kernel theorems,
the certificates and the Rust package are derived from it by the locked
LexLean compiler, and nothing in this repository implements the model in any
other language (R1).

## The model

`mnist-12-int8.onnx` (CNTK-trained, quantized with Intel Neural Compressor;
`model/authorities.toml`): uint8 activations with zero point 0, int8 weights,
int32 biases, about 6,000 parameters.

| Module | What it states |
| --- | --- |
| `Params` | the ten parameter tensors as `int_tensor` artifacts of the ONNX initializers' bytes (`artifacts/`, configured by SHA-256 in `lexlean.toml`; the kernel ties each decoded value to its bytes), and the accessors the network reads them through |
| `Core` | list utilities, the 32-bit fixed-point requantization, dot product, argmax |
| `Conv` | the fused 5×5 convolution: one fold per output row over five padded rows and the accumulator row |
| `Pool` | 2×2 and 3×3 max pooling |
| `Network` | conv1 (1→8), pool, conv2 (8→16), pool, flatten, the ten output codes, `classifyRows : list (list int) → nat` |
| `Model` | the product boundary: a `bytes` image, the contract `ImageClassifier`, the realization `MnistNet`, the evidence, the model `MnistModel`, and the production root `classify` |
| `Image0`, `Conv1Image0`, `Conv2Image0A/B`, `Scenarios` | the kernel-checked forward pass on test image 0 and the unit scenarios |

The contract: precondition `IsImage b` (784 bytes), postcondition
`IsDigit y` (`y ≤ 9`), each with an executable validator and its soundness
theorem. The realization is the network over the image's rows; the evidence
is the universal theorem that every image satisfying the precondition is
classified to a digit (`satisfies_contract`), so the postcondition check is
discharged by proof and only the precondition runs at the boundary:
`Mnist.Model.classify : bytes → result nat contract_violation` refuses an
input that is not 784 bytes and returns a digit otherwise.

Requantization is exact integer arithmetic,
`clamp((acc + bias) · M + 2³¹ + Z · 2³² quot 2³², 0, 255)` with
`M = round(s_in · s_w / s_out · 2³²)` per channel, the multipliers artifacts
derived from the model's scales. Every input-sized loop is a `list_fold`;
there is no indexing of the image and no recursion, and the root admits
exactly the `allocation` and `overflow` effects.

## What is established, and what is measured

Kernel-checked by `lexlean verify` (the attestation the bundle carries):

- each artifact's decoded value is the decoding of its bytes (`W.decoded`),
  and the accessors read what the tensors hold (one kernel, bias, multiplier
  and weight column of each layer, decided);
- the forward pass on test image 0, layer by layer: all eight conv1 channels,
  pool, pad, all sixteen conv2 channels, pool, flatten, all ten output codes,
  argmax, each a decided theorem whose output is the next theorem's input,
  and the compositions `conv1 image0`, `conv2 image0`, `features image0`,
  `codes image0` and `classifyRows image0 = 7` closed by rewriting with them;
- the validators' soundness, and `IsImage b → IsDigit b (MnistNet b)` for
  every `b`;
- certificates A, B and E of the root: source to calculus, calculus to the
  rendered crate, and their composition.

Not established, and said so: that the accumulators never overflow is not a
theorem (the `overflow` effect stays declared and checked at every
operation); the bound on the constants is what the parameters make it, and
the universal lemma over the fused fold is future work.

Measured, and gated by `cargo test` (`MN-05`): the committed package returns
onnxruntime 1.29.0's class on all 10,000 test images (98.92%, 108 errors)
and refuses a 783-byte input. Measured and only reported (`model/ledger.toml`,
`docs/evidence/head-to-head.md`): 8.7 ms per image on one 2-vCPU
container against 66 µs for onnxruntime; where the time goes and what
LexLean #84 does about it is `docs/evidence/rendering.md`.

## The product

`production/request.json` is the `lexlean/production-request/1` that names
the root, the package `mnist` and the `rust-std` target, pinned to the source,
semantic and lock identities. `just model-export` runs
`lexlean production build` on it (which verifies the project) and copies the
bundle to `dist/`: the package `dist/packages/rust-std/mnist` (a safe Rust
2021 library with no dependencies exporting `run(Bytes) -> R<Result<u64,
(bool, bool)>>`), its provenance, the attestation, the certificates, the
lowered program, the crate, and `result.json` binding all of it.
`lexlean production verify-artifact dist` replays the bundle without the
source, Lean or the network (`MN-04`).

## Layout

| Path | Role |
| --- | --- |
| `src/Mnist/*.lex.tex` | the model (above) |
| `artifacts/` | the parameter tensors, content-addressed |
| `lexlean.toml`, `lexlean.lock` | the LexLean project (language 1.2), its artifact sources and its lock |
| `production/request.json` | the production export request |
| `dist/` | the certified bundle the request produces |
| `fixtures/mnist/` | the MNIST test set and onnxruntime's predictions (authorities) |
| `prismpm.toml` | the PrismPM project |
| `model/`, `features/suites/` | the conformance register, authorities, ledger and scenarios |
| `docs/evidence/` | the measurements |

## Gates

`just vv` is the acceptance gate. `just model-verify` runs the LexLean side
alone (`lock --check`, `fmt --check --all`, `check --all`, `build --all`,
`verify --all`); `just model-export` rebuilds `dist/`. The conformance tests
`MN-01`..`MN-06` drive the `lexlean` command and the committed bundle from
`cargo test`; `.github/workflows/model.yml` runs them with LexLean built at
the pinned commit.

This repository is instantiated from
[UOR-Foundation/template](https://github.com/UOR-Foundation/template) at
`1bea460bac6ea50bae53a7eeb674589d7900e6cb`. The template is not yet bound to a
public PrismPM SDK release, so `prismpm.lock` and `template.lock` are absent
and the SDK gates wait on that binding, as in the template itself; PrismPM
packaging of a language-1.2 project waits on PrismPM reconciling to LexLean
main (its #184 and #186).

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
