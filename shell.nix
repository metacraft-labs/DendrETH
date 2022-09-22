{pkgs}:
with pkgs; let
  nodejs = nodejs-18_x;
  llvm = llvmPackages_13;
  corepack = callPackage ./libs/nix/corepack-shims {inherit nodejs;};
  nim-wasm = callPackage ./libs/nix/nim-wasm {inherit llvm;};
in
  mkShell {
    packages = [
      # For priting the direnv banner
      figlet

      # For formatting Nix files
      alejandra

      # For an easy way to launch all required blockchain simulations
      # and tailed log files
      tmux
      tmuxinator

      # Node.js dev environment for unit tests
      nodejs
      corepack

      # For WebAssembly unit-testing
      wasm3 # wasmer is currently broken on macOS ARM

      # Foor finalization of the output and it also provides a
      # 15% size reduction of the generated .wasm files.
      binaryen

      metacraft-labs.circom
      nlohmann_json
      python38
      gmp
      nasm
      libsodium

      # For some reason, this is used by make when compiling the
      # Circom tests on macOS even when we specify CC=clang below:
      gcc

      # Used for building the Nim beacon light client to WebAssembly
      emscripten

      # Used for Nim compilations and for building node_modules
      # Please note that building native node bindings may require
      # other build tools such as gyp, ninja, cmake, gcc, etc, but
      # we currently don't seem to have such dependencies
      llvm.clang

      # llvm.llvm
      # llvm.lld
      ldc
      nim
      nim-wasm
    ];

    shellHook = ''
      export NODE_OPTIONS="--experimental-vm-modules"
      export PATH="$PATH:$PWD/node_modules/.bin";
      export CC=clang
      export LOCAL_NIM_LIB="$PWD/vendor/nim/lib"

      figlet "DendrETH"
    '';
  }
