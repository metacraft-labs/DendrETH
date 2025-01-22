{
  lib,
  nodejs,
  python3,
  sqlite,
  callPackage,
  stdenvNoCC,
  ...
}:
let
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
    "thirdparty/typescript/libs/redis-work-queue"
  ];

  yarnDepsSrc =
    with lib.fileset;
    unions [
      yarnPlugins
      (fileFilter (file: builtins.elem file.name yarnFilenames) root)
    ];

  typeScriptSrc =
    with lib.fileset;
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
      (root + /thirdparty/typescript/libs/redis-work-queue)
    ];

  yarnProject = callPackage ./yarn-project.generated.nix { inherit nodejs; } {
    src =
      with lib.fileset;
      toSource {
        inherit root;
        fileset = yarnDepsSrc;
      };
    overrideAttrs = oldAttrs: {
      dontFixup = true;
      buildInputs = oldAttrs.buildInputs ++ [
        python3
        sqlite
      ];
    };
  };

  finalProject = stdenvNoCC.mkDerivation {
    name = "input-fetchers";
    src =
      with lib.fileset;
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

      # Install executables listed in workspaces' package.json as "bin"
      for w in ${toString workspaces}; do
        installWorkspace "$w"
      done

      rm -rf ".yarn"/{plugins,sdk}
    '';

    doInstallCheck = true;
    installCheckPhase = ''
      join_arr() {
        local IFS="$1"
        shift
        echo "$*"
      }

      # NOTE: Set placeholder values for environment variables for bins using
      # ECS to pass. The module expects the following set of environment
      # variables to be set when imported
      export ECS_REGION="placeholder"
      export ECS_CLUSTER="placeholder"
      export ECS_TASKDEF="placeholder"
      export ECS_CONTAINER="placeholder"
      export ECS_SUBNETS="placeholder"

      set +e

      echo "Executing check phase"

      failing_scripts=()

      for bin in $out/bin/*; do
        bin_name=$(basename $bin)

        echo "Testing \"$bin_name --help\"..."

        stderr=$($bin --help 2>&1 >/dev/null)
        exit_code=$?

        if [[ $exit_code -ne 0 ]]; then
          failing_scripts+=($bin_name)

          echo "\`$bin_name --help\` failed. Make sure it supports \`--help\`
          stderr:
            $stderr"
        fi
      done

      if [ ''${#failing_scripts[@]} -ne 0 ]; then
        echo "Scripts failed: ''${failing_scripts[@]}"
        exit 1
      fi
    '';
  };
in
finalProject
