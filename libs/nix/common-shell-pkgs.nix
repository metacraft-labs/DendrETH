{
  pkgs,
  rust-stable,
}:
with pkgs; let
  llvm = llvmPackages_14;
  emscripten = metacraft-labs.emscripten;
  nim = pkgs.nim1;
  nim-wasm = callPackage ./nim-wasm {inherit llvm emscripten nim;};
  python-with-my-packages = python3.withPackages (ps:
    with ps; [
      py-ecc
      setuptools
      supervisor
    ]);
in
  [
    # For priting the direnv banner
    figlet

    # For formatting Nix files
    alejandra

    # For an easy way to launch all required blockchain simulations
    # and tailed log files
    tmux
    tmuxinator
    # Node.js dev environment for unit tests
    nodejs_20
    corepack_20

    # For WebAssembly unit-testing
    wasm3 # wasmer is currently broken on macOS ARM

    # Foor finalization of the output and it also provides a
    # 15% size reduction of the generated .wasm files.
    binaryen

    (
      pkgs.writeShellScriptBin "circom" ''
        ${metacraft-labs.circom}/bin/circom -l "$CIRCOM_LIB" "$@"
      ''
    )

    nlohmann_json
    python-with-my-packages
    gmp
    nasm
    libsodium

    redis

    b3sum

    # For some reason, this is used by make when compiling the
    # Circom tests on macOS even when we specify CC=clang below:
    gcc

    ccls

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

    go

    rustup
    rust-stable
    # Developer tool to help you get up and running quickly with a new Rust
    # project by leveraging a pre-existing git repository as a template.
    cargo-generate

    docker-client
    curl
    python3
  ]
  ++ lib.optionals (stdenv.isx86_64) [
    metacraft-labs.rapidsnark
  ]
  ++ lib.optionals (stdenv.isLinux && stdenv.isx86_64) [
    nim # Compiling Nim 1.6.8 is currently broken on macOS/M1
    nim-wasm

    # EOS
    metacraft-labs.leap
    metacraft-labs.eos-vm
    metacraft-labs.cdt
    # A basic Cosmos SDK app to host WebAssembly smart contracts
    metacraft-labs.wasmd
    metacraft-labs.rapidsnark-server
  ]
