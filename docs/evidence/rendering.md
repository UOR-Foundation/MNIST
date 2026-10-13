# Rendering: where the 28.7 ms goes, and the ceiling of the current calculus

A measurement (R2) behind `OPEN-03`. It asks one question: of the certified
crate's time, how much is the graph and how much is the way `rust-std` renders
it? The answer is the specification filed as
[LexLean #81](https://github.com/UOR-Foundation/LexLean/issues/81).

## Method

The input is the certified crate of `verification.json` (sha256
`ebacfe87…`), 168,238 bytes of generated Rust. Each variant applies a
mechanical rewrite to that text — a regular expression over the lowered
function bodies, or a replacement of one runtime item — and nothing else. The
graph is untouched; `lexlean` is not re-run; each variant is compiled by the
same `rustc` and driven by the same driver as `head-to-head.md`. Every variant
is accepted only if its predictions equal the certified crate's on every image
tried and its work counter reads the same 39,944 steps on image 0. The
rewritten crates are test text, not part of this repository.

The five rewrites are cumulative:

| | Rewrite | What it replaces |
| --- | --- | --- |
| A | a projection reads one field of the product | `{ let (h, _) = v.clone(); h }`, a clone of the whole product per field read |
| B | `List<T>` as a shared reversed buffer (`Rc<RefCell<Vec<T>>>` + length): O(1) `uncons`, `cons` pushes in place when the list owns the buffer's end | `Rc` cons cells: an allocation per `cons`, two `Rc` clones per `uncons` |
| C | a projected product binding that does not escape is a borrow | a clone of the field |
| D | `list_fold` is a loop over `uncons` with the closure by reference | a recursive driver function cloning the closure and the rest of the list per element |
| E | list buffers start with capacity 32 | growth by reallocation |

## Results

2,000 images each, one 2-vCPU container, single thread, 100% of predictions
equal to the certified crate's (and so to onnxruntime int8's), 39,944 steps.

| Variant | ms/image | of certified |
| --- | --- | --- |
| certified `rust-std` crate | 27.8 | 1.00 |
| + A | 19.3 | 0.69 |
| + B | 9.9 | 0.36 |
| + C | 7.5 | 0.27 |
| + D | 5.5 | 0.20 |
| + E | 4.8 | 0.17 |

Instruction counts (callgrind, three images) locate the remainder. At B the
checked integer arithmetic of the forward pass is 5% of executed instructions;
at D it is 8%, with the rest in by-value movement of the ~260-byte fold state
through the closure's `apply`, in the construction of the returned product,
and in allocation. A plain scalar loop over contiguous arrays does the same
790k multiply-accumulates in about 1 ms; onnxruntime's SIMD int8 kernels in
66 µs.

## What this establishes

The cost of the certified crate is the rendering, by a factor of about six,
and each of the five rules is a change on the machine side of certificate B:
the calculus, the semantic identity, the eligibility verdict and the counted
work are the same before and after. That is the content of LexLean #81. The
graph in `src/Mnist/` is the one that will be re-rendered when it lands; this
repository changes nothing to receive it.
