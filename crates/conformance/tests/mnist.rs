//! `MN-01`..`MN-06`: the MNIST model graph, through the LexLean compiler the SDK supplies, and
//! the certified package bundle the repository ships.
//!
//! These tests drive the public `lexlean` command on the committed project and read what it
//! writes, or build and run the committed bundle's package; they implement nothing of the model
//! (R1) and skip nothing (R4). `lexlean` is resolved from `PATH`, which inside the SDK
//! devcontainer is the locked compiler.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the production bundle is committed (`lexlean production build` output, copied verbatim).
const BUNDLE: &str = "dist";
/// The production root the bundle exports, and the package it exports it as.
const ROOT: &str = "Mnist.Model.classify";
const PACKAGE: &str = "mnist";

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

/// The value of a top-level or nested string field in canonical JSON text, read textually so the
/// tests carry no JSON dependency: the first `"<key>":"<value>"` after `after`.
fn field(text: &str, after: &str, key: &str) -> String {
    let from = text
        .find(after)
        .unwrap_or_else(|| panic!("`{after}` absent"));
    let needle = format!("\"{key}\":\"");
    let start = text[from..]
        .find(&needle)
        .unwrap_or_else(|| panic!("`{key}` absent after `{after}`"))
        + from
        + needle.len();
    text[start..].chars().take_while(|c| *c != '"').collect()
}

fn result_json(root: &Path) -> String {
    std::fs::read_to_string(root.join(BUNDLE).join("result.json")).expect("dist/result.json")
}

#[test]
fn conformance_mn_01() {
    let root = root();
    lexlean(&root, &["lock", "--check"]);
    lexlean(&root, &["fmt", "--check", "--all"]);
    let out = lexlean(&root, &["check", "--all"]);
    assert!(out.starts_with("checked 12 modules"), "{out}");
}

#[test]
fn conformance_mn_02() {
    let root = root();
    let out = lexlean(&root, &["build", "--all"]);
    let build_id = out
        .trim()
        .rsplit('/')
        .next()
        .expect("`built N modules at <path>`")
        .to_string();
    let record = root
        .join(".lexlean/build")
        .join(build_id)
        .join("production/Mnist/Model.eligibility.json");
    let text = std::fs::read_to_string(&record).expect("eligibility record of the build");
    assert!(text.contains(&format!("\"root\":\"{ROOT}\"")), "{record:?}");
    assert!(
        text.contains("\"declared_effects\":[\"allocation\",\"overflow\"]"),
        "classify must admit exactly allocation and overflow: {record:?}"
    );
    assert!(
        text.contains("\"status\":\"eligible\",\"target\":\"rust-std\""),
        "classify must be eligible for rust-std: {record:?}"
    );
    // The statement says the package's folds call themselves (LexLean #105). When LexLean renders
    // them as loops this assertion fails, and that failure is the signal to restore the stronger
    // statement that the runtime closure contains no recursion.
    let lib = std::fs::read_to_string(
        root.join(BUNDLE)
            .join("packages/rust-std")
            .join(PACKAGE)
            .join("src/lib.rs"),
    )
    .expect("the committed package");
    let calling_themselves = self_calling_functions(&lib);
    assert!(
        !calling_themselves.is_empty(),
        "no generated function of the package calls itself: restore MN-02's claim of no recursion"
    );
}

/// The generated functions (`pub fn f<N>(`, each starting a line) whose body calls the function
/// itself. A body runs to the next line that starts another top-level item.
fn self_calling_functions(lib: &str) -> Vec<String> {
    let starts: Vec<(usize, &str)> = lib
        .match_indices("\npub fn f")
        .filter_map(|(at, _)| {
            let rest = &lib[at + "\npub fn ".len()..];
            let name_len = 1 + rest[1..].chars().take_while(char::is_ascii_digit).count();
            (name_len > 1 && rest[name_len..].starts_with('(')).then(|| (at, &rest[..name_len]))
        })
        .collect();
    let mut found = Vec::new();
    for (index, (at, name)) in starts.iter().enumerate() {
        let header_end = at + "\npub fn ".len() + name.len() + 1;
        let end = lib[header_end..]
            .find("\npub ")
            .or_else(|| lib[header_end..].find("\nfn "))
            .map_or(lib.len(), |offset| header_end + offset);
        let end = starts
            .get(index + 1)
            .map_or(end, |(next, _)| end.min(*next));
        let body = &lib[header_end..end];
        let call = format!("{name}(");
        let calls_itself = body.match_indices(&call).any(|(offset, _)| {
            body[..offset]
                .chars()
                .next_back()
                .is_none_or(|before| !before.is_ascii_alphanumeric() && before != '_')
        });
        if calls_itself {
            found.push((*name).to_string());
        }
    }
    found
}

#[test]
fn conformance_mn_03() {
    let root = root();
    let (source, semantic) = identities(&lexlean(&root, &["check", "--all"]));
    let result = result_json(&root);
    assert_eq!(
        field(&result, "\"source\":{", "source_id"),
        source,
        "the bundle's result names the committed source identity"
    );
    assert_eq!(
        field(&result, "\"source\":{", "semantic_id"),
        semantic,
        "the bundle's result names the committed semantic identity"
    );
    // The request pins the same identities, so a bundle built from another source is refused
    // before Lean runs.
    let request = std::fs::read_to_string(root.join("production/request.json")).expect("request");
    assert!(
        request.contains(&source) && request.contains(&semantic),
        "the request pins the identities"
    );
}

#[test]
fn conformance_mn_04() {
    let root = root();
    let stdout = lexlean(&root, &["production", "verify-artifact", BUNDLE]);
    let result = result_json(&root);
    let result_id = field(&result, "{", "result_id");
    assert!(
        stdout.contains(&format!(
            "replayed result {result_id}: 1 outputs, bindings verified"
        )),
        "the replay verifies the bindings of the committed result: {stdout}"
    );
    assert!(
        result.contains(&format!("\"root\":\"{ROOT}\"")),
        "the bundle exports {ROOT}"
    );
    assert_eq!(
        field(&result, "\"certification\":{", "status"),
        "kernel-checked",
        "the export is kernel-checked: certificates A, B and E replayed"
    );
    for role in ["a", "b", "e"] {
        assert!(
            result.contains(&format!("\"role\":\"{role}\"")),
            "the result carries certificate {role}"
        );
    }
}

/// An IDX file (`fixtures/mnist/*.gz`): magic, dimensions, then the bytes.
fn idx(root: &Path, name: &str, dimensions: usize) -> Vec<u8> {
    let compressed = std::fs::File::open(root.join("fixtures/mnist").join(name)).expect("fixture");
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(compressed)
        .read_to_end(&mut bytes)
        .expect("a gzip IDX file");
    bytes.split_off(4 + 4 * dimensions)
}

fn predictions(root: &Path, name: &str) -> Vec<u8> {
    std::fs::read_to_string(root.join("fixtures/mnist").join(name))
        .expect("prediction fixture")
        .lines()
        .map(|line| line.trim().parse().expect("a digit"))
        .collect()
}

#[test]
fn conformance_mn_05() {
    let root = root();
    let package = root.join(BUNDLE).join("packages/rust-std").join(PACKAGE);
    let lib = std::fs::read_to_string(package.join("src/lib.rs")).expect("the committed package");
    assert!(lib.contains("pub fn run("), "the package exports `run`");
    // The harness: every test image through the export, the class compared with onnxruntime's.
    let dir = tempfile::Builder::new()
        .prefix("mnist-agreement-")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("harness dir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        format!(
            "[package]\nname = \"agreement\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\n{PACKAGE} = {{ path = {:?} }}\n\n[profile.release]\nopt-level = 3\n",
            package.display()
        ),
    )
    .expect("harness manifest");
    std::fs::write(
        dir.path().join("src/main.rs"),
        format!(
            r#"use std::io::Write;
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let images = std::fs::read(&args[1]).unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let start = std::time::Instant::now();
    for image in images.chunks_exact(784) {{
        let digit = match {PACKAGE}::run({PACKAGE}::Bytes::lit(image)) {{
            Ok(Ok(digit)) => digit as i64,
            Ok(Err(_)) => -1,
            Err(_) => -2,
        }};
        writeln!(out, "{{digit}}").unwrap();
    }}
    let refused = match {PACKAGE}::run({PACKAGE}::Bytes::lit(&images[..783])) {{ Ok(Err(_)) => true, _ => false }};
    eprintln!("per_image_us={{}} refused_short_input={{refused}}", start.elapsed().as_micros() / (images.len() / 784) as u128);
    assert!(refused, "a 783-byte input is refused by the precondition check");
}}
"#
        ),
    )
    .expect("harness source");
    let images = idx(&root, "t10k-images-idx3-ubyte.gz", 3);
    assert_eq!(images.len(), 10_000 * 784);
    std::fs::write(dir.path().join("images.u8"), &images).expect("images");
    let build = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "--release", "--quiet"])
        .current_dir(dir.path())
        .env("CARGO_TARGET_DIR", dir.path().join("target"))
        .env_remove("RUSTFLAGS")
        .output()
        .expect("cargo runs");
    assert!(
        build.status.success(),
        "the committed package does not build:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let ran = Command::new(dir.path().join("target/release/agreement"))
        .arg(dir.path().join("images.u8"))
        .arg(dir.path().join("predictions.txt"))
        .output()
        .expect("the harness runs");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    eprintln!("{}", String::from_utf8_lossy(&ran.stderr).trim());
    let ours: Vec<i64> = std::fs::read_to_string(dir.path().join("predictions.txt"))
        .expect("predictions")
        .lines()
        .map(|line| line.parse().expect("an integer"))
        .collect();
    let onnxruntime = predictions(&root, "onnxruntime-1.29.0-mnist-12-int8-top1.txt");
    let labels = idx(&root, "t10k-labels-idx1-ubyte.gz", 1);
    assert_eq!(ours.len(), 10_000);
    let disagreements: Vec<usize> = (0..10_000)
        .filter(|&i| ours[i] != i64::from(onnxruntime[i]))
        .collect();
    assert!(
        disagreements.is_empty(),
        "the certified package and onnxruntime int8 disagree on {} images: {:?}",
        disagreements.len(),
        &disagreements[..disagreements.len().min(20)]
    );
    let errors = (0..10_000)
        .filter(|&i| ours[i] != i64::from(labels[i]))
        .count();
    assert_eq!(
        errors, 108,
        "108 of 10,000 test images are misclassified (98.92%)"
    );
}

#[test]
fn conformance_mn_06() {
    let root = root();
    let config = std::fs::read_to_string(root.join("lexlean.toml")).expect("lexlean.toml");
    let authorities =
        std::fs::read_to_string(root.join("model/authorities.toml")).expect("authorities");
    let mut artifacts = 0;
    for entry in std::fs::read_dir(root.join("artifacts")).expect("artifacts/") {
        let path = entry.expect("entry").path();
        let bytes = std::fs::read(&path).expect("artifact bytes");
        let digest = sha256_hex(&bytes);
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            config.contains(&format!(
                "sha256 = \"{digest}\"\npath = \"artifacts/{name}\""
            )),
            "{name}: lexlean.toml configures this artifact under its digest"
        );
        assert!(
            authorities.contains(&digest),
            "{name}: model/authorities.toml records the digest {digest}"
        );
        artifacts += 1;
    }
    assert_eq!(
        artifacts, 10,
        "the ten parameter artifacts of mnist-12-int8"
    );
}

/// The theorems of `Mnist.Facts` (MN-07): every fact the module states.
const FACTS: &[&str] = &[
    "multipliers_positive",
    "fc_multipliers_distinct",
    "conv1_accumulator_range",
    "conv2_accumulator_range",
    "conv1_stored_pairs",
    "conv2_stored_pairs",
    "conv1_extreme_pairing_safe",
    "conv2_channel_0_extreme_pairing_safe",
    "conv2_channel_1_extreme_pairing_safe",
    "conv2_channel_2_extreme_pairing_safe",
    "conv2_channel_3_extreme_pairing_safe",
    "conv2_channel_4_extreme_pairing_safe",
    "conv2_channel_5_extreme_pairing_safe",
    "conv2_channel_6_extreme_pairing_safe",
    "conv2_channel_7_extreme_pairing_safe",
    "conv2_channel_8_extreme_pairing_safe",
    "conv2_channel_9_extreme_pairing_safe",
    "conv2_channel_10_extreme_pairing_safe",
    "conv2_channel_11_extreme_pairing_safe",
    "conv2_channel_12_extreme_pairing_safe",
    "conv2_channel_13_extreme_pairing_safe",
    "conv2_channel_14_extreme_pairing_safe",
    "conv2_channel_15_extreme_pairing_safe",
];

#[test]
fn conformance_mn_07() {
    let root = root();
    let out = lexlean(&root, &["verify", "src/Mnist/Facts.lex.tex"]);
    let attestation_id: String = out
        .split("attestation ")
        .nth(1)
        .unwrap_or_else(|| panic!("verify names its attestation: {out}"))
        .chars()
        .take_while(char::is_ascii_hexdigit)
        .collect();
    let attestation = std::fs::read_to_string(
        root.join(".lexlean/verified")
            .join(&attestation_id)
            .join("attestation.json"),
    )
    .expect("the attestation verify wrote");
    for fact in FACTS {
        let name = format!("\"name\":\"Mnist.Facts.{fact}\"");
        let at = attestation
            .find(&name)
            .unwrap_or_else(|| panic!("{fact} is in the attestation"));
        // A declaration's record ends with its result: `"result":"ok"` when the kernel checked
        // it and its observed axioms are exactly those its policy admits.
        let result = field(&attestation[at..], &name, "result");
        assert_eq!(
            result, "ok",
            "{fact} is checked by the kernel within its axiom policy"
        );
    }
    let source = std::fs::read_to_string(root.join("src/Mnist/Facts.lex.tex")).expect("Facts");
    assert_eq!(
        source.matches("\"kind\":\"theorem\"").count(),
        FACTS.len(),
        "Mnist.Facts states exactly the facts MN-07 names"
    );
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
