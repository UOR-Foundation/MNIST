# Benchmark plan: the certified package against a fixed competitor field

This plan is sealed before any run it governs (R2, `M10` of #3, issue #12). A run is
recorded as an `open` row of `model/ledger.toml` that names this file's SHA-256; a
result never becomes a claim. Changing the plan means a new file with its own
digest, never an edit of this one after a run cites it.

## What is measured

- **Latency.** Wall time per image, one thread, images run one after another, over
  the whole image set.
- **Agreement.** Top-1 class of every image, compared between every implementation
  and the certified package; the ten output codes where an implementation exposes
  them.

Throughput with several threads is not measured: the certified package is
single-threaded, and a multi-threaded figure would compare scheduling, not the
classifier.

## Image set

`MNIST-T10K` (`model/authorities.toml`): the 10,000 test images under
`fixtures/mnist`, fed as raw 0–255 bytes. The int8 model's input has scale 1.0 and
zero point 0, so raw bytes are its intended input (`docs/evidence/head-to-head.md`).

## Implementations

| Implementation | Role | Pinned by |
| --- | --- | --- |
| The committed package `dist/packages/rust-std/mnist`, `run(Bytes)` | Subject | `dist/result.json` |
| onnxruntime 1.29.0, CPU execution provider, default session options, one intra-op thread, `mnist-12-int8.onnx` | Competitor and agreement oracle | `ONNX-MNIST-12-INT8` |
| onnxruntime 1.29.0, CPU execution provider, graph optimization level `ORT_ENABLE_ALL`, one intra-op thread, `mnist-12-int8.onnx` | Competitor | `ONNX-MNIST-12-INT8` |
| onnxruntime 1.29.0, XNNPACK execution provider, one thread, `mnist-12-int8.onnx` | Competitor | `ONNX-MNIST-12-INT8` |
| A hand-specialized C implementation of the same integer arithmetic | Competitor | an authority row naming its URL and SHA-256 before its first run |

Competitors are measurements and oracles, never authorities for this repository's
claims (C9 of #3). The C implementation and every timing driver live outside this
repository, because nothing here implements the model in another language (R1);
each is cited by URL and SHA-256 in `model/authorities.toml` before a run uses it.

## Hardware

- The 2-vCPU Linux container the existing head-to-head was taken on. Each run
  records the CPU model, flags and kernel from `/proc/cpuinfo` and `uname -a`.
- Each further target is added when LexLean models it (UOR-Foundation/LexLean#119);
  which targets come first is UOR-Foundation/LexLean#109.

## Procedure

1. Build each implementation in release mode with the toolchain its pin names.
2. Load all 10,000 images into memory before timing.
3. Run one full pass that is discarded, then five timed passes. Each pass times the
   whole loop with a monotonic clock; per-image time is the pass time divided by
   10,000.
4. Report the median of the five passes, and their minimum and maximum.
5. Run every implementation in the same session on the same machine, one after
   another, with no other workload started by the harness.

## Agreement checks

Before any timing is recorded:

- every implementation's top-1 class is compared with the certified package's on all
  10,000 images, and every disagreement is listed by image index;
- the certified package must refuse a 783-byte input, as MN-05 requires.

An implementation that disagrees on any image is still timed, and its disagreements
are reported beside its time.

## Recording

Each run adds one `open` row to `model/ledger.toml` with `sample_size = 10000`,
naming this file's SHA-256, the commit of every implementation, the hardware record
and the five pass times of each implementation.
