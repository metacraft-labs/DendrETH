{
  lib,
  craneLib,
  fetchFromGitHub,
  pkg-config,
  openssl,
  ...
}: let
  src-path = ../../../beacon-light-client/plonky2/crates;

  sharedAttrs = {
    version = "0.1.0";
    nativeBuildInputs = [pkg-config];
    pname = "circuit-executables";
    buildInputs = [openssl];
    src = src-path;
    postUnpack = ''
      cd $sourceRoot/circuit_executables
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
