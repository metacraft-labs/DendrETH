{
  description = "DendrETH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    flake-utils.url = github:numtide/flake-utils;
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development";
    mcl-blockchain.inputs.nixpkgs.follows = "nixpkgs";
    mcl-blockchain.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    mcl-blockchain,
  }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "DendrETH";
      shell = ./shell.nix;
      preOverlays = [mcl-blockchain.overlays.default];
    };
}
