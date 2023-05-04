{
  lib,
  pkgs,
  ...
}: let
  inherit (pkgs) callPackage fetchFromGitHub;
  inherit (builtins) attrNames filter map readDir;
  inherit (lib) pipe hasSuffix removeSuffix;

  llvm = pkgs.llvmPackages_14;
  emscripten = pkgs.metacraft-labs.emscripten;

  nim-wasm = callPackage ./nim-wasm {inherit llvm emscripten;};
  buildNimWasmProgram = callPackage ./build-nim-wasm-module.nix {inherit nim-wasm;};

  nimDeps = {
    bncurve = fetchFromGitHub {
      owner = "metacraft-labs";
      repo = "nim-bncurve";
      rev = "8136ebdc515df4a3f269eba5f05c625ae742df83";
      hash = "sha256-SZQ7XBy5Ugh/l/XyvOB5wJWWaOpKD+xscDjjh9c/59k=";
    };
    nimcrypto = fetchFromGitHub {
      owner = "cheatfate";
      repo = "nimcrypto";
      rev = "24e006df85927f64916e60511620583b11403178";
      hash = "sha256-KHCMS3ElOS0V3cBmsAuRZGivz8bq5WLUKZOHpg7/Kgg=";
    };
  };

  wasmModules = {
    verify-cosmos = buildNimWasmProgram {
      name = "verify-cosmos";
      srcFile = "verify";
      srcPath = ../../contracts/cosmos/verifier/lib/nim/verify;
      buildInputs = [];
      extraArgs = "--d:lightClientCosmos --path:${nimDeps.bncurve} --path:${nimDeps.nimcrypto}";
    };
  };
in {
  inherit (wasmModules) verify-cosmos;
}
