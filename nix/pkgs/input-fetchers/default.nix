{
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

  yarnFilenames = [
    "yarn.lock"
    "package.json"
    ".yarnrc.yml"
    "tsconfig.json"
  ];

  npmDepsSrc = with lib.fileset;
    toSource {
      inherit root;
      fileset = unions [
        (fileFilter (file: (builtins.elem file.name yarnFilenames) || file.hasExt "ts") root)
        yarnPlugins
        (fileFilter (file: file.hasExt "json") plonky2)
      ];
    };

  yarnProject = callPackage ./yarn-project.generated.nix {inherit nodejs;} {
    src = npmDepsSrc;
  };
  inherit (yarnProject) cacheDrv project;
in
  project.overrideAttrs (oldAttrs: {
    name = "input-fetchers";
    buildInputs = oldAttrs.buildInputs ++ [python3 sqlite];
    # buildPhase = ''
    #   yarn workspace @dendreth/balance-verification build:tsc
    # '';
  })
