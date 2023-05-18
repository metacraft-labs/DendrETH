{
  lib,
  pkgs,
  ...
}: let
  inherit (pkgs) callPackage fetchFromGitHub runCommand stdenv clangStdenv;
  inherit (builtins) attrNames filter map readDir;
  inherit (lib) pipe hasSuffix removeSuffix;

  llvm = pkgs.llvmPackages_14;
  emscripten = pkgs.metacraft-labs.emscripten;

  compile_circuit = circuit:
    stdenv.mkDerivation rec {
      name = "${circuit}_circuit";
      src = ../..;
      buildInputs = [pkgs.metacraft-labs.circom];
      buildPhase = ''
        cd beacon-light-client/circom/circuits
        ls ../../../vendor
        exit
        cd beacon-light-client/circom/scripts/${circuit}
        circom "${circuit}".circom --O1 --r1cs --sym --c --output $out
      '';
    };
  circuits = {
    aggregate_bitmask_circuit = compile_circuit "aggregate_bitmask";
  };
in
  circuits
