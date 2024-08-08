{
  lib,
  craneLib,
  fetchFromGitHub,
  pkg-config,
  openssl,
  ...
}: let
  inherit (lib) fileset;

  root = ../../../beacon-light-client/plonky2;
  cargoDepsRoot = root + /crates;

  cargoDepsSrc = fileset.toSource {
    root = cargoDepsRoot;
    fileset =
      fileset.fileFilter (
        file:
          builtins.elem file.name [
            "Cargo.toml"
            "Cargo.lock"
          ]
      )
      cargoDepsRoot;
  };

  src = fileset.toSource {
    root = root;
    fileset = fileset.unions [
      (root + /common_config.json)
      (root + /kv_db_constants.json)
      (fileset.intersection root cargoDepsRoot)
    ];
  };

  sharedAttrs = rec {
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    cargoLock = "${src}/Cargo.lock";
    cargoToml = "${src}/Cargo.toml";
    buildInputs = [openssl];
    src = cargoDepsSrc;
  };

  brokenTests = lib.concatStringsSep " --skip " ["test_ssz_num_from_bits"];
  slowTests = lib.concatStringsSep " --skip " ["test_deposit_accumulator_diva_leaf_circuit_different_pubkeys" "test_deposit_accumulator_diva_leaf_circuit_is_dummy" "test_deposit_accumulator_diva_leaf_circuit_valid" "test_deposit_accumulator_diva_leaf_circuit_wrong_balances_root" "test_deposit_accumulator_diva_leaf_circuit_wrong_commitment_mapper_branch" "test_deposit_accumulator_diva_leaf_circuit_wrong_validator" "test_deposit_accumulator_diva_leaf_circuit_wrong_validator_gindex"];

  cargoArtifacts = craneLib.buildDepsOnly sharedAttrs;
in
  craneLib.buildPackage (
    sharedAttrs
    // {
      inherit cargoArtifacts src;
      sourceRoot = "source/crates";
      cargoTestExtraArgs = "-- --skip ${brokenTests} --skip ${slowTests}";
    }
  )
