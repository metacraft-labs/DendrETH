{
  lib,
  craneLib,
  fetchFromGitHub,
  pkg-config,
  openssl,
  ...
}: let
  eth2-tests = fetchFromGitHub {
    owner = "ethereum";
    repo = "eth2.0-tests";
    rev = "b01e476e90898e0092a848455797cee4e1609eff";
    hash = "sha256-m+4w2wEXn4YZPs9YwrvdW5Lmd6N24ol5+zyJuBXLPKs=";
  };

  src-path = ../../../beacon-light-client/plonky2;

  sharedAttrs = {
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    buildInputs = [openssl];
    src = src-path;
    cargoLock = "${src-path}/crates/Cargo.lock";
    cargoToml = "${src-path}/crates/Cargo.toml";
    postUnpack = ''
      cd $sourceRoot/crates
      sourceRoot="."
    '';
    preCheck = ''
      pwd
      ls -la
      cp -r ${eth2-tests} $sourceRoot/
      sed -i 's|../../../../vendor/eth2.0-tests|eth2.0-tests|' "beacon-light-client/plonky2/crates/circuits/src/utils/circuit/mod.rs"
    '';
  };

  cargoArtifacts = craneLib.buildDepsOnly sharedAttrs;
in
  craneLib.buildPackage (
    sharedAttrs
    // {
      inherit cargoArtifacts;
    }
  )
