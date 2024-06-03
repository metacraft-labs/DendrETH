{
  pkgs,
  rust-stable,
}:
with pkgs; let
  light-client = callPackage ./light-client {};
  shell-pkgs = import ./common-shell-pkgs.nix {inherit pkgs rust-stable;};
in
  mkShell {
    packages =
      shell-pkgs
      ++ [
        light-client
      ];

    shellHook = ''
      set -e

      export NODE_OPTIONS="--experimental-vm-modules --max-old-space-size=16384"
      export PATH="$PATH:$PWD/node_modules/.bin";
      export CC=clang
      export LOCAL_NIM_LIB="$PWD/vendor/nim/lib"
      export LOCAL_HARDHAT_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

      figlet "DendrETH"
    '';
  }
