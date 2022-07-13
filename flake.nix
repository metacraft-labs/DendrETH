{
  description = "DendrETH";

  # Opt into `nix-blockchain-development`'s substituter (binary cache).
  # `nixConfig` settings are not transitive so every user of a flake with a
  # custom binary cache must manually include its `nixConfig` settings for
  # substituters and trusted public keys:
  nixConfig = {
    extra-substituters = "https://nix-blockchain-development.cachix.org";
    extra-trusted-public-keys = "nix-blockchain-development.cachix.org-1:Ekei3RuW3Se+P/UIo6Q/oAgor/fVhFuuuX5jR8K/cdg=";
  };

  inputs = {
    # To ensure all packages from mcl-blockchain will be fetched from its
    # binary cache we need to ensure that we use exact same commit hash of the
    # inputs below. If we didn't, we may either:
    # * end up with multiple copies of the same package from nixpkgs
    # * be unable to use the binary cache, since the packages there where
    #   using different versions of their dependencies from nixpkgs
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development";
    nixpkgs.follows = "mcl-blockchain/nixpkgs";
    flake-utils.follows = "mcl-blockchain/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    mcl-blockchain,
    rust-overlay,
  }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "DendrETH";
      shell = ./shell.nix;
      config = {
        permittedInsecurePackages = [
          # wasm3 is insecure if used to execute untrusted third-party code
          # however, since we're using it for development, these problems do not
          # affect us.
          # Marked as insecure: https://github.com/NixOS/nixpkgs/pull/192915
          "wasm3-0.5.0"
        ];
      };
      preOverlays = [
        mcl-blockchain.overlays.default (import ./libs/nix/overlay.nix)
        rust-overlay.overlays.default
      ];
      overlay = ./overlay.nix;
    };
}
