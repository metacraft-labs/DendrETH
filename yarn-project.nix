# This file is generated by running "yarn install" inside your project.
# Manual changes might be lost - proceed with caution!

{ lib, stdenv, nodejs, git, cacert, fetchurl, writeShellScript, writeShellScriptBin }:
{ src, overrideAttrs ? null, ... } @ args:

let

  yarnBin = ./.yarn/releases/yarn-4.1.1.cjs;

  cacheFolder = ".yarn/cache";
  lockfile = ./yarn.lock;

  # Call overrideAttrs on a derivation if a function is provided.
  optionalOverride = fn: drv:
    if fn == null then drv else drv.overrideAttrs fn;

  # Simple stub that provides the global yarn command.
  yarn = writeShellScriptBin "yarn" ''
    exec '${nodejs}/bin/node' '${yarnBin}' "$@"
  '';

  # Common attributes between Yarn derivations.
  drvCommon = {
    # Make sure the build uses the right Node.js version everywhere.
    buildInputs = [ nodejs yarn ];
    # All dependencies should already be cached.
    yarn_enable_network = "0";
    # Tell node-gyp to use the provided Node.js headers for native code builds.
    npm_config_nodedir = nodejs;
  };

  # Comman variables that we set in a Nix build, but not in a Nix shell.
  buildVars = ''
    # Make Yarn produce friendlier logging for automated builds.
    export CI=1
    # Tell node-pre-gyp to never fetch binaries / always build from source.
    export npm_config_build_from_source=true
    # Disable Nixify plugin to save on some unnecessary processing.
    export yarn_enable_nixify=false
  '';

  cacheDrv = stdenv.mkDerivation {
    name = "yarn-cache";
    buildInputs = [ yarn git cacert ];
    buildCommand = ''
      cp --reflink=auto --recursive '${src}' ./src
      cd ./src/
      ${buildVars}
      HOME="$TMP" yarn_enable_global_cache=false yarn_cache_folder="$out" \
        yarn nixify fetch
      rm $out/.gitignore
    '';
    outputHashMode = "recursive";
    outputHash = "sha512-997JYdQe5iW9rBM3sKDUAw89eoufLwHKiEE2filYg8lEl+yMXRe3bsj2EDRbP9b+NDsvgjgYotfQOc1frGF8cA==";
  };

  # Create a derivation that builds a module in isolation.
  mkIsolatedBuild = { pname, version, reference, locators ? [] }: stdenv.mkDerivation (drvCommon // {
    inherit pname version;
    dontUnpack = true;

    configurePhase = ''
      ${buildVars}
      unset yarn_enable_nixify # plugin is not present
    '';

    buildPhase = ''
      mkdir -p .yarn/cache
      cp --reflink=auto --recursive ${cacheDrv}/* .yarn/cache/

      echo '{ "dependencies": { "${pname}": "${reference}" } }' > package.json
      install -m 0600 ${lockfile} ./yarn.lock
      export yarn_global_folder="$TMP"
      export yarn_enable_global_cache=false
      export yarn_enable_immutable_installs=false
      yarn
    '';

    installPhase = ''
      unplugged=( .yarn/unplugged/${pname}-*/node_modules/* )
      if [[ ! -e "''${unplugged[@]}" ]]; then
        echo >&2 "Could not find the unplugged path for ${pname}"
        exit 1
      fi

      mv "$unplugged" $out
    '';
  });

  # Main project derivation.
  project = stdenv.mkDerivation (drvCommon // {
    inherit src;
    name = "DendrETH";

    configurePhase = ''
      ${buildVars}

      # Copy over the Yarn cache.
      rm -fr '${cacheFolder}'
      mkdir -p '${cacheFolder}'
      cp --reflink=auto --recursive ${cacheDrv}/* '${cacheFolder}/'

      # Yarn may need a writable home directory.
      export yarn_global_folder="$TMP"

      # Ensure global cache is disabled. Cache must be part of our output.
      touch .yarnrc.yml
      sed -i -e '/^enableGlobalCache/d' .yarnrc.yml
      echo 'enableGlobalCache: false' >> .yarnrc.yml

      # Some node-gyp calls may call out to npm, which could fail due to an
      # read-only home dir.
      export HOME="$TMP"

      # running preConfigure after the cache is populated allows for
      # preConfigure to contain substituteInPlace for dependencies as well as the
      # main project. This is necessary for native bindings that maybe have
      # hardcoded values.
      runHook preConfigure

      # Copy in isolated builds.
      echo 'injecting build for msgpackr-extract'
      yarn nixify inject-build \
        "msgpackr-extract@npm:3.0.2" \
        ${isolated."msgpackr-extract@npm:3.0.2"} \
        ".yarn/unplugged/msgpackr-extract-npm-3.0.2-93e8773fad/node_modules/msgpackr-extract"
      echo 'injecting build for bufferutil'
      yarn nixify inject-build \
        "bufferutil@npm:4.0.8" \
        ${isolated."bufferutil@npm:4.0.8"} \
        ".yarn/unplugged/bufferutil-npm-4.0.8-8005ed6210/node_modules/bufferutil"
      echo 'injecting build for utf-8-validate'
      yarn nixify inject-build \
        "utf-8-validate@npm:5.0.10" \
        ${isolated."utf-8-validate@npm:5.0.10"} \
        ".yarn/unplugged/utf-8-validate-npm-5.0.10-93e9b6f750/node_modules/utf-8-validate"
      echo 'injecting build for keccak'
      yarn nixify inject-build \
        "keccak@npm:3.0.4" \
        ${isolated."keccak@npm:3.0.4"} \
        ".yarn/unplugged/keccak-npm-3.0.4-a84763aab8/node_modules/keccak"
      echo 'injecting build for secp256k1'
      yarn nixify inject-build \
        "secp256k1@npm:4.0.3" \
        ${isolated."secp256k1@npm:4.0.3"} \
        ".yarn/unplugged/secp256k1-npm-4.0.3-b4e9ce065b/node_modules/secp256k1"
      echo 'injecting build for bcrypto'
      yarn nixify inject-build \
        "bcrypto@npm:5.5.2" \
        ${isolated."bcrypto@npm:5.5.2"} \
        ".yarn/unplugged/bcrypto-npm-5.5.2-f3838b92ce/node_modules/bcrypto"
      echo 'injecting build for blake-hash'
      yarn nixify inject-build \
        "blake-hash@npm:2.0.0" \
        ${isolated."blake-hash@npm:2.0.0"} \
        ".yarn/unplugged/blake-hash-npm-2.0.0-c63b9a2c2d/node_modules/blake-hash"
      echo 'injecting build for bcrypt'
      yarn nixify inject-build \
        "bcrypt@npm:5.0.1" \
        ${isolated."bcrypt@npm:5.0.1"} \
        ".yarn/unplugged/bcrypt-npm-5.0.1-6815be1cfe/node_modules/bcrypt"
      echo 'injecting build for keccak'
      yarn nixify inject-build \
        "keccak@npm:3.0.1" \
        ${isolated."keccak@npm:3.0.1"} \
        ".yarn/unplugged/keccak-npm-3.0.1-9f0a714d5c/node_modules/keccak"
      echo 'injecting build for bufferutil'
      yarn nixify inject-build \
        "bufferutil@npm:4.0.5" \
        ${isolated."bufferutil@npm:4.0.5"} \
        ".yarn/unplugged/bufferutil-npm-4.0.5-88cc521694/node_modules/bufferutil"
      echo 'injecting build for keccak'
      yarn nixify inject-build \
        "keccak@npm:3.0.2" \
        ${isolated."keccak@npm:3.0.2"} \
        ".yarn/unplugged/keccak-npm-3.0.2-6e9dec8765/node_modules/keccak"
      echo 'injecting build for leveldown'
      yarn nixify inject-build \
        "leveldown@npm:6.1.0" \
        ${isolated."leveldown@npm:6.1.0"} \
        ".yarn/unplugged/leveldown-npm-6.1.0-c2d7a4250d/node_modules/leveldown"
      echo 'injecting build for utf-8-validate'
      yarn nixify inject-build \
        "utf-8-validate@npm:5.0.7" \
        ${isolated."utf-8-validate@npm:5.0.7"} \
        ".yarn/unplugged/utf-8-validate-npm-5.0.7-88d731f8ad/node_modules/utf-8-validate"
      echo 'injecting build for redis-commander'
      yarn nixify inject-build \
        "redis-commander@npm:0.8.0" \
        ${isolated."redis-commander@npm:0.8.0"} \
        ".yarn/unplugged/redis-commander-npm-0.8.0-4f9397694b/node_modules/redis-commander"
      echo 'running yarn install'

      # Run normal Yarn install to complete dependency installation.
      yarn install --immutable --immutable-cache

      runHook postConfigure
    '';

    buildPhase = ''
      runHook preBuild
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      # Move the package contents to the output directory.
      if grep -q '"workspaces"' package.json; then
        # We can't use `yarn pack` in a workspace setup, because it only
        # packages the outer workspace.
        mkdir -p "$out/libexec"
        mv $PWD "$out/libexec/$name"
      else
        # - If the package.json has a `files` field, only files matching those patterns are copied
        # - Otherwise all files are copied.
        yarn pack --out package.tgz
        mkdir -p "$out/libexec/$name"
        tar xzf package.tgz --directory "$out/libexec/$name" --strip-components=1

        cp --reflink=auto .yarnrc* "$out/libexec/$name"
        cp --reflink=auto ${lockfile} "$out/libexec/$name/yarn.lock"
        cp --reflink=auto --recursive .yarn "$out/libexec/$name"

        # Copy the Yarn linker output into the package.
        cp --reflink=auto .pnp.* "$out/libexec/$name"
      fi

      cd "$out/libexec/$name"

      # Invoke a plugin internal command to setup binaries.
      mkdir -p "$out/bin"
      yarn nixify install-bin $out/bin

      runHook postInstall
    '';

    passthru = {
      inherit nodejs;
      yarn-freestanding = yarn;
      yarn = writeShellScriptBin "yarn" ''
        exec '${yarn}/bin/yarn' --cwd '${overriddenProject}/libexec/${overriddenProject.name}' "$@"
      '';
    };
  });

  overriddenProject = optionalOverride overrideAttrs project;

isolated."msgpackr-extract@npm:3.0.2" = optionalOverride (args.overrideMsgpackrExtractAttrs or null) (mkIsolatedBuild { pname = "msgpackr-extract"; version = "3.0.2"; reference = "npm:3.0.2"; });
isolated."bufferutil@npm:4.0.8" = optionalOverride (args.overrideBufferutilAttrs or null) (mkIsolatedBuild { pname = "bufferutil"; version = "4.0.8"; reference = "npm:4.0.8"; });
isolated."utf-8-validate@npm:5.0.10" = optionalOverride (args.overrideUtf8ValidateAttrs or null) (mkIsolatedBuild { pname = "utf-8-validate"; version = "5.0.10"; reference = "npm:5.0.10"; });
isolated."keccak@npm:3.0.4" = optionalOverride (args.overrideKeccakAttrs or null) (mkIsolatedBuild { pname = "keccak"; version = "3.0.4"; reference = "npm:3.0.4"; });
isolated."secp256k1@npm:4.0.3" = optionalOverride (args.overrideSecp256k1Attrs or null) (mkIsolatedBuild { pname = "secp256k1"; version = "4.0.3"; reference = "npm:4.0.3"; });
isolated."bcrypto@npm:5.5.2" = optionalOverride (args.overrideBcryptoAttrs or null) (mkIsolatedBuild { pname = "bcrypto"; version = "5.5.2"; reference = "npm:5.5.2"; });
isolated."blake-hash@npm:2.0.0" = optionalOverride (args.overrideBlakeHashAttrs or null) (mkIsolatedBuild { pname = "blake-hash"; version = "2.0.0"; reference = "npm:2.0.0"; });
isolated."bcrypt@npm:5.0.1" = optionalOverride (args.overrideBcryptAttrs or null) (mkIsolatedBuild { pname = "bcrypt"; version = "5.0.1"; reference = "npm:5.0.1"; });
isolated."keccak@npm:3.0.1" = optionalOverride (args.overrideKeccakAttrs or null) (mkIsolatedBuild { pname = "keccak"; version = "3.0.1"; reference = "npm:3.0.1"; });
isolated."bufferutil@npm:4.0.5" = optionalOverride (args.overrideBufferutilAttrs or null) (mkIsolatedBuild { pname = "bufferutil"; version = "4.0.5"; reference = "npm:4.0.5"; });
isolated."keccak@npm:3.0.2" = optionalOverride (args.overrideKeccakAttrs or null) (mkIsolatedBuild { pname = "keccak"; version = "3.0.2"; reference = "npm:3.0.2"; });
isolated."leveldown@npm:6.1.0" = optionalOverride (args.overrideLeveldownAttrs or null) (mkIsolatedBuild { pname = "leveldown"; version = "6.1.0"; reference = "npm:6.1.0"; });
isolated."utf-8-validate@npm:5.0.7" = optionalOverride (args.overrideUtf8ValidateAttrs or null) (mkIsolatedBuild { pname = "utf-8-validate"; version = "5.0.7"; reference = "npm:5.0.7"; });
isolated."redis-commander@npm:0.8.0" = optionalOverride (args.overrideRedisCommanderAttrs or null) (mkIsolatedBuild { pname = "redis-commander"; version = "0.8.0"; reference = "npm:0.8.0"; });
in overriddenProject
