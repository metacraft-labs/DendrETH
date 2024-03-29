{
  lib,
  nodejs,
  python3,
  sqlite,
  callPackage,
  ...
}: let
  overrideDep = old: {
    buildInputs = old.buildInputs ++ [python3 sqlite];
  };
  project =
    callPackage ../../../yarn-project.nix {
      inherit nodejs;
    } {
      src = lib.cleanSource ../../..;
      overrideBcryptoAttrs = overrideDep;
      overrideBufferutilAttrs = overrideDep;
      overrideClassicLevelAttrs = overrideDep;
      overrideMsgpackrExtractAttrs = overrideDep;
      overrideUtf8ValidateAttrs = overrideDep;
      overrideLeveldownAttrs = overrideDep;
      overrideBcryptAttrs = overrideDep;
      overrideGetBalancesInputAttrs = overrideDep;
    };
in
  project.overrideAttrs (oldAttrs: {
    name = "get-changed-validators";
    buildInputs = oldAttrs.buildInputs ++ [python3 sqlite];
    buildPhase = ''
      yarn build-plonky-2
    '';
    postInstall = ''
      mkdir -p $out/bin
      cp -r beacon-light-client/plonky2/mock_data $out/bin'';
  })
