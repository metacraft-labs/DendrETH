{
  pkgs,
  lib,
  fetchFromGitHub,
  rustPlatform,
}:
with pkgs;
  rustPlatform.buildRustPackage rec {
    pname = "snowbridge-parachain";
    version = "0.0.1";
    src = fetchFromGitHub {
      owner = "snowfork";
      repo = "snowbridge";
      sha256 = "sha256-bIaPwxcWaTO60L4Qur5gmqEmVHfCtaS5c6rVqCL1dpg=";
      rev = "a65d9118dd4b2277eb7a8513c6f9d7273f277fc2";
    };

    CARGO_INCREMENTAL = 0;
    RUST_BACKTRACE = 1;
    RUSTFLAGS = "-C debuginfo=1";
    SKIP_WASM_BUILD = 1;
    PROTOC = "${protobuf}/bin/protoc";
    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

    # buildAndTestSubdir = "source/parachain";
    sourceRoot = "source/parachain";

    cargoHash = "sha256-l1vL2ZdtDRxSGvP0X/l3nMw8+6WF67KPutJEzUROjg8=";

    cargoLock = let
      fixupLockFile = path: (builtins.readFile path);
    in {
      lockFileContents = fixupLockFile ./Cargo.lock;
      outputHashes = {
        "amcl-0.3.0" = "sha256-JbmfWh/r9PU1XSurvEzy54kIZGaH2AHUQGy47a39Upg=";
        "beefy-gadget-4.0.0-dev" = "sha256-UOFJ/fZuaNtF3v8nTvOwE6y9RDSM00IZ2ZJTPK7Eiw0=";
        "bp-header-chain-0.1.0" = "sha256-bmQAHZy+jkmCtFNKUwiQc/cLp/7dUCUCThcpndGXbis=";
        "cumulus-client-cli-0.1.0" = "sha256-Nbr27xqFOCgtvaKmmQYHczpUbmDrgwzTRV8X0sM0k/U=";
        "ethabi-decode-1.3.3" = "sha256-Now4iQ/X2WzUkNjfn5V2PdjcV4vsABq8UVPehXSWRCo=";
        "ethash-0.5.0" = "sha256-oo9hW83EtjX2HAMMVqJ8N8VSP+58SRU+gjW0cXB0cxo=";
        "ssz-rs-0.8.0" = "sha256-fXhVuhT0NthvScU+l/al41XAyH3cK0gHbGJ7coDUzK8=";
      };
    };
    postPatch = ''
      cp ${./Cargo.lock} Cargo.lock
    '';

    buildNoDefaultFeatures = true;
    doCheck = false;
    buildFeatures = [
      "snowblink-native"
      # "snowbase-native"
      # "snowbridge-native"
      # "rococo-native"
    ];

    nativeBuildInputs = [
      rust-bin.stable."1.58.1".default
      protobuf
      clang_11
    ];
    buildInputs = [
    ];

    meta = with lib; {
      homepage = "https://github.com/Snowfork/snowbridge";
      platforms = platforms.linux;
    };
  }
