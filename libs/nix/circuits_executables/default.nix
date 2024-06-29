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

  cargoArtifacts = craneLib.buildDepsOnly sharedAttrs;
in
  craneLib.buildPackage (
    sharedAttrs
    // {
      inherit cargoArtifacts;
    }
  )
