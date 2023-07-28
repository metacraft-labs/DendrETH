{
  pkgs,
  rust-stable,
  rust-nightly,
}:
with pkgs; let
  shell-pkgs = import ./libs/nix/common-shell-pkgs.nix {inherit pkgs rust-stable;};
in
  mkShell {
    packages = [rust-nightly] ++ shell-pkgs;

    nativeBuildInputs = [pkg-config openssl];

    shellHook = ''
      set -e

      export NODE_OPTIONS="--experimental-vm-modules"
      export PATH="$PATH:$PWD/node_modules/.bin";
      export CC=clang
      export LOCAL_NIM_LIB="$PWD/vendor/nim/lib"
      export LOCAL_HARDHAT_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

      if [ -f .env ]; then
        set -a
        source .env
        set +a
      fi

      # scripts/check-user-env-file-contents.sh

      figlet "DendrETH"
    '';
  }
