{
  pkgs,
  lib,
  fetchFromGitHub,
  rustPlatform,
  rust,
}:
with pkgs;
  rustPlatform.buildRustPackage rec {
    pname = "dylint";
    version = "2.0.0";
    src = fetchgit {
      url = "http://github.com/trailofbits/dylint.git";
      sha256 = "sha256-3LZ+ys7ePhumOQJExZISrwt6fLxPDt45TEhkF/hG1Ew=";
      rev = "v${version}";
    };

    CARGO_INCREMENTAL = 0;
    RUST_BACKTRACE = 1;
    RUSTFLAGS = "-C debuginfo=1";
    PROTOC = "${protobuf}/bin/protoc";
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
    PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
    RUSTUP_TOOLCHAIN = "1.58.1";

    cargoHash = "sha256-l1vL2ZdtDRxSGvP0X/l3nMw8+61F67K2utJEzUROjg8=";

    cargoLock = {
      lockFile = "${src}/Cargo.lock";
      outputHashes = {
      };
    };

    doCheck = false;

    nativeBuildInputs = [
      rust-bin.stable."1.58.1".default
      clang_11
      openssl.dev
      pkg-config
    ] ++ lib.optionals (stdenv.isDarwin) [
      darwin.apple_sdk.frameworks.Security
    ];

    buildInputs = [
    ];

    meta = with lib; {
      homepage = "http://github.com/trailofbits/dylint";
      targets = platforms.linux;
    };
  }
