{pkgs ? import <nixpkgs> {}}: let
  project =
    pkgs.callPackage ../../../yarn-project.nix {
      nodejs = pkgs.nodejs-18_x;
    } {
      src = pkgs.lib.cleanSource ../../..;
      overrideBcryptoAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideBufferutilAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideClassicLevelAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideMsgpackrExtractAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideUtf8ValidateAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideLeveldownAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideBcryptAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
      overrideGetBalancesInputAttrs = old: {
        buildInputs = old.buildInputs ++ [pkgs.python3 pkgs.sqlite];
      };
    };
in
  project.overrideAttrs (oldAttrs: {
    name = "get-balances-input";
    buildInputs = oldAttrs.buildInputs ++ [pkgs.python3 pkgs.sqlite];
    buildPhase = ''
      yarn build-plonky-2
    '';
    postInstall = ''
      mkdir -p $out/bin
      cp -r beacon-light-client/plonky2/get_balances_input/mock_data $out/bin'';
  })
