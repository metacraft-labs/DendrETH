{
  lib,
  craneLib,
  fetchFromGitHub,
  ...
}: let
  src-path = ../../../beacon-light-client/plonky2;
  cargoArtifacts = craneLib.buildDepsOnly {
    pname = "plonky2";
    version = "0.1.0";

    src = fetchFromGitHub {
      owner = "metacraft-labs";
      repo = "plonky2";
      rev = "12402078a460c41cd11013d065367c8e25bb8478";
      hash = "sha256-uPfN65vlWh92Se8muhrO071WNaAGI+PUOY4x1syvspU=";
    };
  };
in
  craneLib.buildPackage rec {
    pname = "circuits_executables";
    version = "0.1.0";

    inherit cargoArtifacts;
    src = src-path;
    cargoLock = "${src-path}/circuits_executables/Cargo.lock";
    cargoToml = "${src-path}/circuits_executables/Cargo.toml";

    postUnpack = ''
      cd $sourceRoot/circuits_executables
      sourceRoot="."
    '';
  }