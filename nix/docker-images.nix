{
  pkgs,
  nix2container,
  self'
}:
with pkgs; let
  nodejs = nodejs-18_x;
  corepack = metacraft-labs.corepack-shims;
  python-with-my-packages = python3.withPackages (ps:
    with ps; [
      supervisor
    ]);

  yarn_install_pkgs = [
    bash
    coreutils
    gnumake
    gnused
    stdenv.cc
    nodejs
    corepack
    python-with-my-packages
  ];

  runtime_packages =
    yarn_install_pkgs
    ++ [
      self'.packages.light-client
      gmp
      nasm
      libsodium
      fish
      redis
      curl
      b3sum
      metacraft-labs.rapidsnark-server
    ];

  docker-image-yarn =
    nix2container.buildImage
    {
      name = "dendreth-relay-yarn";
      tag = "latest";
      copyToRoot = pkgs.buildEnv {
        name = "image-root";
        paths = yarn_install_pkgs;
        pathsToLink = ["/bin"];
      };
    };

  docker-image-all =
    nix2container.buildImage
    {
      name = "dendreth-relay";
      tag = "latest";
      copyToRoot = pkgs.buildEnv {
        name = "image-root";
        paths = runtime_packages;
        pathsToLink = ["/bin"];
      };
    };
in {
  inherit yarn_install_pkgs runtime_packages docker-image-yarn docker-image-all;
}
