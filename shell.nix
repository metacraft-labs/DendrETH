{pkgs}:
with pkgs; let
  nodejs = nodejs-18_x;
  corepack = callPackage ./nix/corepack-shims {inherit nodejs;};
  llvm = llvmPackages_14;
in
  mkShell {
    buildInputs = [
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

      nim

      llvm.lld
      llvm.clang-unwrapped
      llvm.llvm
      # Foor finalization of the output and it also provides a
      # 15% size reduction of the generated .wasm files.
      binaryen

      ldc

      nix-tree

      metacraft-labs.solana
      criterion # needed for solana
    ];

    shellHook = ''
      export NODE_OPTIONS="--experimental-vm-modules"
      export PATH="$PATH:$PWD/node_modules/.bin";
      export CC=clang
      export C_INCLUDE_PATH="${nim-unwrapped}/nim/lib:${glibc.dev}/include:${criterion.dev}/include"
      export LIBRARY_PATH="${glibc.dev}/lib"
      figlet "DendrETH"
    '';
  }
