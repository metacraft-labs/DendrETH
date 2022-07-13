{
  description = "DendrETH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-utils.url = github:numtide/flake-utils;

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "DendrETH";
      shell = ./shell.nix;
      overlay = ./overlay.nix;
      preOverlays = [(import rust-overlay)];
    };
}
