{ inputs, ... }:
{
  perSystem =
    {
      lib,
      inputs',
      pkgs,
      system,
      ...
    }:
    let
      inherit (inputs'.mcl-nixos-modules.legacyPackages)  rust-stable rust-latest;
    in
    {
      devShells.default =
        with pkgs;
        let
          shell-pkgs = import ./libs/nix/common-shell-pkgs.nix { inherit pkgs rust-stable; };
        in
        mkShell {
          packages = [ rust-latest ] ++ shell-pkgs;

          nativeBuildInputs = [
            pkg-config
            openssl
          ];

          shellHook = ''
            set -e

            export NODE_OPTIONS="--experimental-vm-modules"
            export CC=clang
            export LOCAL_NIM_LIB="$PWD/vendor/nim/lib"
            export CIRCOM_LIB="$(find $PWD/.yarn/unplugged -maxdepth 1 -type d -name 'circomlib-*')/node_modules/circomlib/circuits"
            export LOCAL_HARDHAT_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

            export GIT_ROOT="$(git rev-parse --show-toplevel)"

            if [ -f .env ]; then
              set -a
              source .env
              set +a
            fi

            # scripts/check-user-env-file-contents.sh

            # Set up the environment for the Solidity compiler
            # ./scripts/config_solidity_import_mapping.sh

            figlet "DendrETH"
          '';
        };
    };
}
