{
  lib,
  craneLib,
  fetchFromGitHub,
  pkg-config,
  openssl,
  ...
}: let
  src-path = ../../../beacon-light-client/plonky2;

  sharedAttrs = {
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    buildInputs = [openssl];
    src = src-path;
    cargoLock = "${src-path}/crates/Cargo.lock";
    cargoToml = "${src-path}/crates/Cargo.toml";
    cargoTestExtraArgs = "--no-run";
    postUnpack = ''
      cd $sourceRoot/crates
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
