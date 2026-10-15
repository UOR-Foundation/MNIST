# Planted defects

A gate is evidence only after a planted defect shows it can fail (AGENTS.md, gate
quality). Each row records the defect, the failing command, its diagnostic, and the
restored result.

| Gate | Planted defect | Failing command and diagnostic | Restored |
| --- | --- | --- | --- |
| `conformance_mn_02` | The committed package's generated functions rewritten so that none calls itself, as if LexLean rendered every fold as a loop (UOR-Foundation/LexLean#105 fixed) | `cargo test -p repo-conformance --test mnist -- conformance_mn_02` panics: `no generated function of the package calls itself: restore MN-02's claim of no recursion` | The committed `dist/packages/rust-std/mnist/src/lib.rs` restored byte for byte; the test passes and finds 15 self-calling functions (`f9`, `f12`, `f23`, `f32`, `f42`, `f44`, `f47`, `f62`, `f65`, `f81`, `f83`, `f93`, `f96`, `f98`, `f103`) |
| `conformance_mn_07` | One stated value of `Mnist.Facts` changed: the greatest conv1 bias plus 255 times its positive taps stated as 210789 instead of 210788 | `cargo test -p repo-conformance --test mnist -- conformance_mn_07` panics: `error[LLV7002]: Lean rejected Mnist.Facts (error): Tactic decide proved that the proposition … is false` | The committed `src/Mnist/Facts.lex.tex` restored; the source identity returns to `34a408b9…`, and verify accepts all 23 theorems |
