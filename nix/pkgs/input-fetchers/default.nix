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
      yarn build-plonky-2
      # CURR_DIR=$(pwd)
      # (cd ${packageJsonFiles}; cp -r --parents . $CURR_DIR/dist)
      # chmod -R 666 $CURR_DIR/dist/
      (cd ${packageJsonFiles} && find . -type f -exec install -Dm666 "{}" "./dist/{}" \;)
    '';
  })
# git ls-files | grep "package.json" | tr '\n' '\0' | xargs -0 -n1 sh -c 'x="$out/libexec/input-fetchers/dist/$1" && mkdir -p "${x%/*}" && cat "$1" > "$x"' -s

