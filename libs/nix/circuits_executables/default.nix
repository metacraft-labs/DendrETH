{
  lib,
  craneLib,
  fetchFromGitHub,
  pkg-config,
  openssl,
  ...
}: let
  src-path = ../../..;

  sharedAttrs = {
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    cargoLock = "${src-path}/beacon-light-client/plonky2/crates/Cargo.lock";
    cargoToml = "${src-path}/beacon-light-client/plonky2/crates/Cargo.toml";
    buildInputs = [openssl];
    src = src-path;
    doChecks = false;
    postUnpack = ''
      cd $sourceRoot/beacon-light-client/plonky2/crates
      sourceRoot="."
    '';
  };

  brokenTests = lib.concatStringsSep " --skip " ["test_ssz_num_from_bits"];
  slowTests = lib.concatStringsSep " --skip " ["test_deposit_accumulator_diva_leaf_circuit_different_pubkeys" "test_deposit_accumulator_diva_leaf_circuit_is_dummy" "test_deposit_accumulator_diva_leaf_circuit_valid" "test_deposit_accumulator_diva_leaf_circuit_wrong_balances_root" "test_deposit_accumulator_diva_leaf_circuit_wrong_commitment_mapper_branch" "test_deposit_accumulator_diva_leaf_circuit_wrong_validator" "test_deposit_accumulator_diva_leaf_circuit_wrong_validator_gindex"];

  cargoArtifacts = craneLib.buildDepsOnly sharedAttrs;
in
  craneLib.buildPackage (
    sharedAttrs
    // {
      inherit cargoArtifacts;
      cargoTestExtraArgs = "-- --skip ${brokenTests} --skip ${slowTests}";
    }
  )
