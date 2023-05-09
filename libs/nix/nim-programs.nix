{
  lib,
  pkgs,
  rustPlatformStable,
  ...
}: let
  inherit (pkgs) callPackage fetchFromGitHub runCommand stdenv clangStdenv;
  inherit (builtins) attrNames filter map readDir;
  inherit (lib) pipe hasSuffix removeSuffix;

  llvm = pkgs.llvmPackages_14;
  emscripten = pkgs.metacraft-labs.emscripten;

  nim-wasm = callPackage ./nim-wasm {inherit llvm emscripten;};
  buildNimWasmProgram = callPackage ./build-nim-wasm-module.nix {inherit nim-wasm;};
  buildNimProgram = callPackage ./build-nim-module.nix {inherit (pkgs) nim;};

  _nimDeps = [
    "${
      # bncurve =
      fetchFromGitHub
      {
        owner = "metacraft-labs";
        repo = "nim-bncurve";
        rev = "8136ebdc515df4a3f269eba5f05c625ae742df83";
        hash = "sha256-SZQ7XBy5Ugh/l/XyvOB5wJWWaOpKD+xscDjjh9c/59k=";
      }
    }"
    "${
      # nimcrypto =
      fetchFromGitHub
      {
        owner = "cheatfate";
        repo = "nimcrypto";
        rev = "24e006df85927f64916e60511620583b11403178";
        hash = "sha256-KHCMS3ElOS0V3cBmsAuRZGivz8bq5WLUKZOHpg7/Kgg=";
      }
    }"
    "${
      # confutils =
      fetchFromGitHub
      {
        owner = "status-im";
        repo = "nim-confutils";
        rev = "a26bfab7e5fb2f9fc018e5d778c169bc05772ee6";
        hash = "sha256-Shf+0HMlWvbB7mqscCR60Cnsd04cUTxo3tSyXcQlOr0=";
      }
    }"
    "${
      # stew =
      fetchFromGitHub
      {
        owner = "status-im";
        repo = "nim-stew";
        rev = "06621a2fcddf01b0231aaa6d18531b0a746b3140";
        hash = "sha256-I/Vka8rVjAfQ6le/d7hNebm0J9+oUCtMVBQSkeRtC7Y=";
      }
    }"
    "${
      # serialization =
      fetchFromGitHub
      {
        owner = "status-im";
        repo = "nim-serialization";
        rev = "493d18b8292fc03aa4f835fd825dea1183f97466";
        hash = "sha256-2nwMg2x1uDl3f4VL66w69EbBOhLdbOStOnML6rgmyoc=";
      }
    }"
    "${
      # faststreams =
      fetchFromGitHub
      {
        owner = "status-im";
        repo = "nim-faststreams";
        rev = "c80701f7d23815fab0b6362569f3195957e57856";
        hash = "sha256-HWcPJWivqZOrcoyvSlsnGpDlzhCdJitEPLsWAExwLtA=";
      }
    }"
    "${
      # ssz_serialization =
      fetchFromGitHub
      {
        owner = "status-im";
        repo = "nim-ssz-serialization";
        rev = "66097b911158d459e5114fabd0998f0b2870f853";
        hash = "sha256-sIey3Vd7kgP4idwAJP4/Q3/VRnvdt43X4GTQzDrJHFg=";
      }
    }"
    "${
      # stint =
      fetchFromGitHub {
        owner = "status-im";
        repo = "nim-stint";
        rev = "27a7608f33a485fb837c9759ddb4a7c874eb7ad2";
        hash = "sha256-f3b64XBtAyzYiO1YVuzXkbWDOfG8KrPUuCFbCkvPM2c=";
      }
    }"
    "${
      # blscurve =
      fetchFromGitHub {
        owner = "status-im";
        repo = "nim-blscurve";
        rev = "d93da7af30a9a2160f68d84c5210a12c4d16df00";
        hash = "sha256-dO2g+ZZ9HS1Fm1ejzwOEQFoFFTJgEjHABuMQyWwxTOI=";
        fetchSubmodules = true;
      }
    }"
    "${
      # json-serialization =
      fetchFromGitHub {
        owner = "status-im";
        repo = "nim-json-serialization";
        rev = "b06b8ca4b177a7d0f74ac602350316d95c2fddd5";
        hash = "sha256-8o3GHjCQfBSkfTxj3NDikqf/vehPvHYRFIlwf4Mpdhg=";
      }
    }"
  ];

  nimDeps = toString (map (x: "--path:${x}") _nimDeps);

  rust-optimizer = rustPlatformStable.buildRustPackage rec {
    name = "rust-optimizer";
    buildInputs = [pkgs.bash];
    cargoHash = "sha256-Cp6929/Y5CARjqBu6rYk+VAemGCok1PjNXnepbr+DOU=";
    src = fetchFromGitHub {
      owner = "CosmWasm";
      repo = "rust-optimizer";
      rev = "ffc496a2a3f2f93b2be4e17446d4ab4265e4d1e1";
      hash = "sha256-92LVuso7p1nnTYvwdXHos5nYoqt/XBBaXxSoJgj7cWE=";
    };
    postInstall = ''
      cd ..
      cp *.sh $out/bin
      chmod +x $out/bin/*.sh
      sed -i "s#/usr/local/bin/build_workspace#$out/build-workspace#" $out/bin/*.sh
      sed -i "s#/bin/ash#/bin/bash#" $out/bin/*.sh
      sed -i "s# -o nounset##" $out/bin/*.sh
    '';
    sourceRoot = "source/build_workspace";
  };

  wasmModules = {
    cosmos-nim-verifier-wasm = buildNimWasmProgram {
      name = "cosmos-nim-verifier-wasm";
      outFileName = "nim_verifier";
      srcFile = "verify";
      src = ../../contracts/cosmos/verifier/lib/nim/verify;
      extraArgs = "--d:lightClientCosmos ${nimDeps}";
    };

    cosmos-nim-light-client-wasm = buildNimWasmProgram {
      name = "cosmos-nim-light-client-wasm";
      outFileName = "light_client";
      srcFile = "light_client_cosmos_wrapper";
      src = ../../contracts/cosmos/light-client/lib/nim;
      extraArgs = "--d:lightClientCosmos ${nimDeps} --path:${../../beacon-light-client/nim}";
    };

    beacon-light-client-wasm = buildNimWasmProgram {
      name = "beacon-light-client";
      srcFile = "beacon-light-client/nim/light_client";
      src = ../..;
      extraArgs = "${nimDeps}";
    };

    beacon-light-client-emmc-wasm = buildNimWasmProgram {
      name = "beacon-light-client-emmc";
      srcFile = "beacon-light-client/nim/light_client";
      src = ../..;
      extraArgs = "-d:emmc ${nimDeps}";
    };

    cosmos-verifier-parse-data = buildNimProgram {
      name = "cosmos-verifier-parse-data";
      srcFile = "tests/cosmosLightClient/helpers/verifier-parse-data-tool/verifier_parse_data";
      src = ../..;
      extraArgs = "--d:nimOldCaseObjects ${nimDeps}";
    };

    cosmos-groth16-verifier = buildNimProgram {
      name = "groth16-verifier";
      srcFile = "libs/nim/nim-groth16-verifier/verify";
      src = ../..;
      extraArgs = "--d:lightClientCosmos ${nimDeps}";
    };

    cosmos-verifier-contract = stdenv.mkDerivation rec {
      name = "cosmos-verifier-contract";
      src = ../../contracts/cosmos/verifier;
      cargoDeps = rustPlatformStable.fetchCargoTarball {
        inherit src;
        name = "${name}";
        sha256 = "sha256-/uNX2t3qaAphj9KiPcchs/qa3zhd2DMhKpJ1JfZdAk4=";
      };

      CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse";
      NIMCACHE_PARENT = "${wasmModules.cosmos-nim-verifier-wasm}";

      buildPhase = ''
        echo buildPhase;
        optimize.sh .
      '';

      installPhase = ''
        mkdir -p $out/lib
        cp ./target/wasm32-unknown-unknown/release/verifier.wasm $out/lib
      '';

      nativeBuildInputs = with pkgs;
        [binaryen cargo glibc rust-optimizer sccache wasmModules.cosmos-nim-verifier-wasm wasmModules.cosmos-verifier-parse-data]
        ++ [
          rustPlatformStable.cargoSetupHook
        ]
        ++ (with rustPlatformStable; [rust.cargo rust.rustc]);
    };

    cosmos-light-client-contract = stdenv.mkDerivation rec {
      name = "cosmos-light-client-contract";
      src = ../../contracts/cosmos/light-client;
      cargoDeps = rustPlatformStable.fetchCargoTarball {
        inherit src;
        name = "${name}";
        sha256 = "sha256-BKUpZMIT83kNHMMGmnLEfScCInIegiUG629bsJi1N9E=";
      };

      CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse";
      NIMCACHE_PARENT = "${wasmModules.cosmos-nim-light-client-wasm}";

      buildPhase = ''
        echo buildPhase;
        optimize.sh .
      '';

      installPhase = ''
        mkdir -p $out/lib
        cp ./target/wasm32-unknown-unknown/release/light_client.wasm $out/lib
      '';

      nativeBuildInputs = with pkgs;
        [binaryen cargo glibc rust-optimizer sccache wasmModules.cosmos-nim-light-client-wasm]
        ++ [
          rustPlatformStable.cargoSetupHook
        ]
        ++ (with rustPlatformStable; [rust.cargo rust.rustc]);
    };
  };
in {
  inherit (wasmModules) cosmos-verifier-parse-data cosmos-verifier-contract cosmos-light-client-contract cosmos-groth16-verifier beacon-light-client-wasm beacon-light-client-emmc-wasm;
}
