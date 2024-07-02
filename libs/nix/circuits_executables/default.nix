{
  lib,
  craneLib,
  pkg-config,
  openssl,
  ...
}: let
  kv_db_constants = ../../../beacon-light-client/plonky2/kv_db_constants.json;

  sharedAttrs = rec {
    src = ../../../beacon-light-client/plonky2/crates;
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    cargoLock = "${src}/Cargo.lock";
    cargoToml = "${src}/Cargo.toml";
    buildInputs = [openssl];
  };

  brokenTests = lib.concatStringsSep " --skip " ["test_ssz_num_from_bits"];
  slowTests = lib.concatStringsSep " --skip " [
    "test_deposit_accumulator_diva_leaf_circuit_different_pubkeys"
    "test_deposit_accumulator_diva_leaf_circuit_is_dummy"
    "test_deposit_accumulator_diva_leaf_circuit_valid"
    "test_deposit_accumulator_diva_leaf_circuit_wrong_balances_root"
    "test_deposit_accumulator_diva_leaf_circuit_wrong_commitment_mapper_branch"
    "test_deposit_accumulator_diva_leaf_circuit_wrong_validator"
    "test_deposit_accumulator_diva_leaf_circuit_wrong_validator_gindex"
  ];

  cargoArtifacts = craneLib.buildDepsOnly sharedAttrs;
in
  craneLib.buildPackage (
    sharedAttrs
    // {
      inherit cargoArtifacts;
      cargoTestExtraArgs = "-- --skip ${brokenTests} --skip ${slowTests}";
      postUnpack = ''
        sed -i 's|../../../kv_db_constants.json|${kv_db_constants}|g' crates/circuit_executables/src/db_constants.rs
      '';
    }
  )
