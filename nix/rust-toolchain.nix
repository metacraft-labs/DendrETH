{inputs, ...}: let
  inherit (inputs.mcl-blockchain.inputs) crane;
in {
  perSystem = {pkgs, ...}: let
    rust = pkgs.rust-bin.nightly."2024-03-28".default.override {
      extensions = ["rust-src" "rust-analyzer"];
    };
    craneLib = (crane.mkLib pkgs).overrideToolchain rust;
  in {
    legacyPackages = {
      rustToolchain = {
        inherit rust craneLib;
      };
    };
  };
}
