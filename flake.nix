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
    flake-parts.follows = "mcl-blockchain/flake-parts";
  };

  outputs = inputs @ {
    self,
    flake-parts,
    nixpkgs,
    mcl-blockchain,
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
        inherit (inputs'.mcl-blockchain.legacyPackages) nix2container rust-stable rustPlatformStable;
        docker-images = import ./libs/nix/docker-images.nix {inherit pkgs nix2container;};
        light-client = pkgs.callPackage ./libs/nix/light-client/default.nix {};
        nim-packages = import ./libs/nix/nim-programs.nix {
          inherit pkgs rustPlatformStable;
          lib = pkgs.lib;
        };
      in {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [
            mcl-blockchain.overlays.default
          ];
          config.permittedInsecurePackages = [
            # wasm3 is insecure if used to execute untrusted third-party code
            # however, since we're using it for development, these problems do not
            # affect us.
            # Marked as insecure: https://github.com/NixOS/nixpkgs/pull/192915
            "wasm3-0.5.0"
          ];
        };
        packages =
          {
            inherit (docker-images) docker-image-yarn;
            inherit light-client;
          }
          // nim-packages
          // pkgs.lib.optionalAttrs (pkgs.hostPlatform.isLinux && pkgs.hostPlatform.isx86_64) {
            inherit (docker-images) docker-image-all;
          };
        devShells.default = import ./shell.nix {inherit pkgs rust-stable;};
        devShells.light-client = import ./libs/nix/shell-with-light-client.nix {inherit pkgs rust-stable;};
      };
    };
}
