{
  pkgs,
  lib,
  fetchFromGitHub,
  rustPlatform,
  rust,
}:
with pkgs;
  rustPlatform.buildRustPackage rec {
    pname = "rust-clippy";
    version = "1.61.0";
    src = fetchgit {
      url = "https://github.com/rust-lang/rust-clippy.git";
      sha256 = "sha256-lq4CwcdosLVTRfbwcNFKUqfkwQQsN92QYPuwWjfke5g=";
      rev = "c995a8b5017766246ac5cfe66baa074eeee3a5a3";
    };

    patches = [./build.rs.patch];

    CARGO_INCREMENTAL = 0;
    RUST_BACKTRACE = 1;
    PROTOC = "${protobuf}/bin/protoc";
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
    PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
    RUSTUP_TOOLCHAIN = "${version}";

    # RUSTC_VERSION = builtins.readFile "${src}/rust-toolchain";
    RUSTFLAGS = "-C debuginfo=1 -Zunstable-options -Zbinary-dep-depinfo";

    cargoHash = "sha256-l1vL2ZdtDRxSGvP0X/l3nMw8+61F67K2utJEzUROjg8=";

    cargoLock = let
      fixupLockFile = path: (builtins.readFile path);
    in {
      lockFileContents = fixupLockFile ./Cargo.lock;
      outputHashes = {
      };
    };

    postPatch = ''
      cp ${./Cargo.lock} Cargo.lock
    '';

    doCheck = false;

    nativeBuildInputs = [
      rust-bin.nightly."2022-03-14".complete
      clang_11
      openssl.dev
      pkg-config
    ];
    buildInputs = [
    ];

    meta = with lib; {
      homepage = "https://github.com/rust-lang/rust-clippy";
      targets = platforms.linux;
    };
  }
