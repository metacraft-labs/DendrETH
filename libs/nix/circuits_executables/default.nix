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
    buildInputs = [openssl];
    src = src-path;
    cargoLock = "${src-path}/circuits_executables/Cargo.lock";
    cargoToml = "${src-path}/circuits_executables/Cargo.toml";
    postUnpack = ''
      cd $sourceRoot/circuits_executables
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
