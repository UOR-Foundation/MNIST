Feature: model

  The MNIST model under src/Mnist: the network, its parameters as artifacts, the contract, the
  realization, the evidence, and the certified package bundle of the root Mnist.Model.classify.

  @MN-01 @build
  Scenario: The MNIST model under src/Mnist is accepted by the locked LexLean compiler: the lock is current, every module is canonical, and all eleven modules link, artifacts, contract, realization, evidence and model included.
    Given the committed lexlean.toml, lexlean.lock, the artifacts and the eleven modules under src/Mnist
    When the suite exercises MN-01
    Then lexlean lock --check, lexlean fmt --check --all and lexlean check --all each exit successfully and eleven modules are checked

  @MN-02 @build
  Scenario: The root Mnist.Model.classify, the model applied under its precondition check, is production-eligible for the rust-std target and admits exactly the allocation and overflow effects; its source has no recursion, but the package renders each list fold as a function that calls itself (LexLean #105), so its stack depth is bounded by the list lengths its 784-byte precondition fixes.
    Given the committed model and the committed bundle's package
    When the suite exercises MN-02
    Then lexlean build writes an eligibility record for Mnist.Model.classify whose declared effects are exactly allocation and overflow and whose target is rust-std, and the package's generated functions include ones that call themselves

  @MN-03 @build
  Scenario: The committed bundle's result and the committed production request name the source and semantic identities of the current model, so the attestation, certificates and package the bundle carries belong to the committed source.
    Given dist/result.json, production/request.json and the committed model
    When the suite exercises MN-03
    Then the source and semantic identities lexlean check reports equal the identities the result names and the request pins

  @MN-04 @build
  Scenario: The committed bundle replays offline through lexlean production verify-artifact with status bindings-verified, exporting Mnist.Model.classify with certification kernel-checked: certificates A, B and E of the root were compiled, replayed and axiom-audited by the pinned Lean.
    Given the committed bundle under dist/
    When the suite exercises MN-04
    Then lexlean production verify-artifact reports bindings-verified and the result names the root with certification kernel-checked

  @MN-05 @build
  Scenario: The committed package, built from the bundle and run over the 10,000 MNIST test images, returns the class onnxruntime 1.29.0 returns for mnist-12-int8.onnx on every image, misclassifies exactly 108 of them, and refuses an input that is not 784 bytes.
    Given dist/packages/rust-std/mnist, the MNIST test set and onnxruntime's predictions under fixtures/mnist
    When the suite exercises MN-05
    Then a harness built against the package agrees with onnxruntime on 10,000 of 10,000 images, counts 108 errors against the labels, and gets a contract violation for a 783-byte input

  @MN-06 @build
  Scenario: Every parameter artifact under artifacts/ is configured in lexlean.toml under its SHA-256 and recorded in model/authorities.toml, so the weights the model decodes are the bytes the authority row names.
    Given artifacts/, lexlean.toml and model/authorities.toml
    When the suite exercises MN-06
    Then each of the ten artifacts' SHA-256 appears as its artifact_source digest and in the authority record
