import LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0
import LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.RustStd
import LexLeanPreservation.Compose
set_option autoImplicit false
set_option maxRecDepth 100000
set_option linter.unusedVariables false
namespace LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.Compose.RustStd

/-- The rendering of the root, invoked on the encoded arguments, realizes
the encoded source result. -/
theorem root (b : ByteArray) (__e_hrep : _root_.LexLeanPreservation.Rust.RepresentableL [(_root_.LexLeanTarget.TargetSyntax.Value.bytes b)]) : ∃ __e_ro, _root_.LexLeanPreservation.Rust.RealizesFn Bool.true (_root_.LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.denote b) __e_ro ∧ _root_.LexLeanPreservation.Rust.RCI _root_.LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.RustStd.krate (_root_.LexLeanPreservation.Rust.fnIdent 0) [(_root_.LexLeanTarget.TargetSyntax.Value.bytes b)] __e_ro :=
  _root_.LexLeanPreservation.Rust.compose (fun __e_n => _root_.LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.RustStd.root __e_n 0) rfl (_root_.LexLeanPreservation.Rust.wtl_cons (_root_.LexLeanPreservation.Rust.wt_bytes b) _root_.LexLeanPreservation.Rust.wtl_nil) rfl rfl (_root_.LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.root b) (_root_.LexLeanPreservation.Rust.rel_ne_stuck _ _)

end LexLeanPreserve.C04d46d7fd36e23950c7b567c8b89ffaa.R0.Compose.RustStd
