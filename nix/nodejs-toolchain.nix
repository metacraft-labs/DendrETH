{...}: {
  perSystem = {pkgs, ...}: let
    nodejs = pkgs.nodejs_24;
    corepack = pkgs.corepack.override {inherit nodejs;};
  in {
    legacyPackages = {
      nodejsToolchain = {
        inherit nodejs corepack;
      };
    };
  };
}
