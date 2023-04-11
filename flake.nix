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
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

    nix2container.url = "github:nlewo/nix2container";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    flake-utils,
    mcl-blockchain,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem = {
        config,
        system,
        pkgs,
        inputs',
        ...
      }: let
        nix2container = inputs'.nix2container.packages.nix2container;
        docker-images = import ./libs/nix/docker-images.nix {inherit pkgs nix2container;};
      in {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [
            mcl-blockchain.overlays.default
            (import ./libs/nix/overlay.nix)
            rust-overlay.overlays.default
          ];
          config.permittedInsecurePackages = [
            # wasm3 is insecure if used to execute untrusted third-party code
            # however, since we're using it for development, these problems do not
            # affect us.
            # Marked as insecure: https://github.com/NixOS/nixpkgs/pull/192915
            "wasm3-0.5.0"
          ];
        };
        packages = {
          inherit (docker-images) docker-image-yarn docker-image-all;
        };
        devShells.default = import ./shell.nix {inherit pkgs;};
        devShells.container = import ./relay/shell.nix {inherit pkgs;};
      };
    };
}
