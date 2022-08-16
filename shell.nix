{pkgs}:
with pkgs; let
  nodejs = nodejs-18_x;
  llvm = llvmPackages_13;
  corepack = callPackage ./nix/corepack-shims {inherit nodejs;};
  nim-wasm = callPackage ./nix/nim-wasm {inherit llvm;};
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

      llvm.clang
      ldc
      nim
      nim-wasm
      python38
    ];

    shellHook = ''
      export NODE_OPTIONS="--experimental-vm-modules"
      export PATH="$PATH:$PWD/node_modules/.bin";
      export CC=clang
      figlet "DendrETH"
    '';
  }
