{
  pkgs,
  lib,
  fetchFromGitHub,
  rustPlatform,
  rust,
}:
with pkgs;
  rustPlatform.buildRustPackage rec {
    pname = "cargo-contract";
    version = "1.4.0";
    src = fetchgit {
      url = "https://github.com/paritytech/cargo-contract";
      sha256 = "sha256-PcrGBThelWgO7oKfUt1RFcV7zAm6nOV0vB+S5+K3HnU=";
      rev = "v${version}";
      leaveDotGit = true;
    };

    patches = [./build.rs.patch];

    CARGO_INCREMENTAL = 0;
    RUST_BACKTRACE = 1;
    RUSTFLAGS = "-C debuginfo=1";
    PROTOC = "${protobuf}/bin/protoc";
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
    SKIP_WASM_BUILD = 1;

    RUSTUP_TOOLCHAIN = "nightly-2022-03-14";

    cargoHash = "sha256-b1nMJQQt3m4DNCKCwEyN/1fbDWGfXQfdWlzeQHCtTtc=";
    # cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    cargoPatches = [./Cargo.patch];

    nativeBuildInputs = [
      # rust-bin.stable."1.58.1".default.override
      # {
      #   extensions = ["rust-src"];
      #   targets = ["wasm32-unknown-unknown-wasm"];
      # }
      rust-bin.nightly."2022-03-14".complete
      clang_11
      dylint
      rust-clippy
    ];

    doCheck = false;

    meta = with lib; {
      homepage = "https://github.com/paritytech/cargo-contract";
      targets = platforms.linux;
    };
  }
