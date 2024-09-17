{
  lib,
  nodejs,
  python3,
  sqlite,
  callPackage,
  stdenvNoCC,
  ...
}: let
  root = ../../..;
  yarnPlugins = root + /.yarn/plugins;

  yarnFilenames = [
    "yarn.lock"
    "package.json"
    ".yarnrc.yml"
  ];

  tsconfigFiles = [
    "tsconfig.json"
    "tsconfig.hardhat.json"
  ];

  workspaces = [
    "beacon-light-client/plonky2/input_fetchers"
    "beacon-light-client/solidity"
    "libs/typescript"
    "relay"
  ];

  yarnDepsSrc = with lib.fileset;
    unions [
      yarnPlugins
      (fileFilter (file: builtins.elem file.name yarnFilenames) root)
    ];

  typeScriptSrc = with lib.fileset;
    unions [
      yarnDepsSrc
      (fileFilter (file: (builtins.elem file.name tsconfigFiles)) root)
      (root + /beacon-light-client/plonky2/input_fetchers)
      (root + /beacon-light-client/plonky2/common_config.json)
      (root + /beacon-light-client/plonky2/kv_db_constants.json)
      (root + /beacon-light-client/solidity)
      (root + /libs/typescript)
      (root + /relay)
      (root + /rollup)
    ];

  yarnProject = callPackage ./yarn-project.generated.nix {inherit nodejs;} {
    src = with lib.fileset;
      toSource {
        inherit root;
        fileset = yarnDepsSrc;
      };
    overrideAttrs = oldAttrs: {
      dontFixup = true;
      buildInputs = oldAttrs.buildInputs ++ [python3 sqlite];
    };
  };

  finalProject = stdenvNoCC.mkDerivation {
    name = "input-fetchers";
    src = with lib.fileset;
      toSource {
        inherit root;
        fileset = typeScriptSrc;
      };
    nativeBuildInputs = yarnProject.buildInputs;
    postUnpack = ''
      dir=${yarnProject}/libexec/DendrETH
      cp --reflink=auto --recursive --no-preserve=all $dir/. /build/source
    '';
    buildPhase = ''
      NODE_OPTIONS="--experimental-import-meta-resolve" yarn build:all
    '';
    dontFixup = true;
    installPhase = ''
      set -x
      dst="$out/libexec/$name"
      mkdir -p "$dst" "$out/bin"
      mv $PWD/{.yarn,.pnp.cjs,.pnp.loader.mjs,.yarnrc.yml,yarn.lock,package.json} "$dst/"

      installWorkspace() {
        local workspace="$1"
        mkdir -p "$dst/$workspace"
        mv "$PWD/$workspace"/{package.json,dist} "$dst/$workspace"
        (
          cd "$dst/$workspace"
          yarn nixify install-bin $out/bin
        )
      }

      for w in ${toString workspaces}; do
        installWorkspace "$w"
      done

      rm -rf ".yarn"/{plugins,sdk}
    '';
  };
in
  finalProject
# project.overrideAttrs (oldAttrs: {
#   name = "input-fetchers";
#   buildInputs = oldAttrs.buildInputs ++ [python3 sqlite];
#   buildPhase = ''
#     yarn build:all
#   '';
# })
