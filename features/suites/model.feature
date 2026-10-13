Feature: model

  The MNIST model graph under src/Mnist and the certified root Mnist.Network.classify.

  @MN-01 @build
  Scenario: The MNIST model graph under src/Mnist is accepted by the locked LexLean compiler: the lock is current, every module is canonical, and every module links.
    Given the committed lexlean.toml, lexlean.lock and the six modules under src/Mnist
    When the suite exercises MN-01
    Then lexlean lock --check, lexlean fmt --check --all and lexlean check --all each exit successfully

  @MN-02 @build
  Scenario: The root Mnist.Network.classify is production-eligible for the rust-std target and admits exactly the allocation and overflow effects, so its runtime closure contains no recursion.
    Given the committed model graph
    When the suite exercises MN-02
    Then lexlean build writes an eligibility record for Mnist.Network.classify whose declared effects are exactly allocation and overflow and whose target is rust-std

  @MN-03 @build
  Scenario: The committed verification record names the source and semantic identities of the current model graph, so the attestation, certificates and crate digest it carries belong to the committed source.
    Given docs/evidence/verification.json and the committed model graph
    When the suite exercises MN-03
    Then the source and semantic identities lexlean check reports equal the identities the record names
