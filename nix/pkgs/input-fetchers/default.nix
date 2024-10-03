{
  pkgs,
  lib,
  nodejs,
  python3,
  sqlite,
  callPackage,
  ...
}: let
  root = ../../..;
  yarnPlugins = root + /.yarn/plugins;
  plonky2 = root + /beacon-light-client/plonky2;
  relay = root + /relay;

  yarnFilenames = [
    "yarn.lock"
    "package.json"
    ".yarnrc.yml"
    "tsconfig.json"
  ];

  packageJsonFiles = with lib.fileset;
    toSource {
      inherit root;
      fileset = fileFilter (file: file.name == "package.json") root;
    };

  npmDepsSrc = with lib.fileset;
    toSource {
      inherit root;
      fileset = unions [
        (fileFilter (file: (builtins.elem file.name yarnFilenames) || file.hasExt "ts") root)
        yarnPlugins
        (fileFilter (file: file.hasExt "json") plonky2)
        (fileFilter (file: file.hasExt "json") relay)
      ];
    };

  yarnProject = callPackage ./yarn-project.generated.nix {inherit nodejs;} {
    src = npmDepsSrc;
  };
in
  yarnProject.overrideAttrs (oldAttrs: {
    name = "input-fetchers";
    buildInputs = oldAttrs.buildInputs ++ [python3 sqlite];
    buildPhase = ''
      yarn tsc -p beacon-light-client/plonky2/input_fetchers/tsconfig.json
      yarn tsc -p relay/tsconfig.json
      CURR_DIR=$(pwd)
      (cd ${packageJsonFiles} && find . -type f -exec install -Dm666 "{}" "$CURR_DIR/dist/{}" \;)
    '';
  })

