//! `MN-01`..`MN-03`: the MNIST model graph, through the LexLean compiler the SDK supplies.
//!
//! These tests drive the public `lexlean` command on the committed project and read what it
//! writes; they implement nothing of the model (R1) and skip nothing (R4). `lexlean` is resolved
//! from `PATH`, which inside the SDK devcontainer is the locked compiler.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

/// Run `lexlean <args>` at the repository root and return its stdout; panic with the diagnostic
/// text on a nonzero exit.
fn lexlean(root: &Path, args: &[&str]) -> String {
    let out = Command::new("lexlean")
        .args(args)
        .current_dir(root)
        .output()
        .expect("lexlean is on PATH inside the SDK");
    assert!(
        out.status.success(),
        "lexlean {} failed:\n{}{}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `lexlean check` reports `checked N modules (source <hex>, semantic <hex>)`.
fn identities(check_output: &str) -> (String, String) {
    let take = |key: &str| -> String {
        let start = check_output
            .find(key)
            .unwrap_or_else(|| panic!("`{key}` absent from: {check_output}"))
            + key.len();
        check_output[start..]
            .chars()
            .take_while(char::is_ascii_hexdigit)
            .collect()
    };
    (take("source "), take("semantic "))
}

#[test]
fn conformance_mn_01() {
    let root = root();
    lexlean(&root, &["lock", "--check"]);
    lexlean(&root, &["fmt", "--check", "--all"]);
    let out = lexlean(&root, &["check", "--all"]);
    assert!(out.starts_with("checked 6 modules"), "{out}");
}

#[test]
fn conformance_mn_02() {
    let root = root();
    let out = lexlean(&root, &["build"]);
    let build_id = out
        .trim()
        .rsplit('/')
        .next()
        .expect("`built N modules at <path>`")
        .to_string();
    let record = root
        .join(".lexlean/build")
        .join(build_id)
        .join("production/Mnist/Network.eligibility.json");
    let text = std::fs::read_to_string(&record).expect("eligibility record of the build");
    // The record is JSON; the assertions below read its two load-bearing fields textually so the
    // test carries no JSON dependency.
    assert!(
        text.contains("\"root\":\"Mnist.Network.classify\""),
        "{record:?}"
    );
    assert!(
        text.contains("\"declared_effects\":[\"allocation\",\"overflow\"]"),
        "classify must admit exactly allocation and overflow: {record:?}"
    );
    assert!(
        text.contains("\"status\":\"eligible\",\"target\":\"rust-std\""),
        "classify must be eligible for rust-std: {record:?}"
    );
}

#[test]
fn conformance_mn_03() {
    let root = root();
    let (source, semantic) = identities(&lexlean(&root, &["check", "--all"]));
    let record = std::fs::read_to_string(root.join("docs/evidence/verification.json"))
        .expect("docs/evidence/verification.json");
    assert!(
        record.contains(&format!("\"source\": \"{source}\"")),
        "the verification record names source {source}"
    );
    assert!(
        record.contains(&format!("\"semantic\": \"{semantic}\"")),
        "the verification record names semantic identity {semantic}"
    );
}
