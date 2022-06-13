{pkgs}:
with pkgs; let
  nodejs = nodejs-16_x;
  corepack = callPackage ./nix/corepack-shims {inherit nodejs;};
in
  mkShell {
    buildInputs = [
      # For priting the direnv banner
      figlet

      # For formatting Nix files
      alejandra

      # For an easy way to launch all required blockchain simulations
      # and tailed log files
      tmux
      tmuxinator

      # Node.js dev environment for unit tests
      nodejs
      corepack

      # For WebAssembly unit-testing
      wasm3 # wasmer is currently broken on macOS ARM
    ];

    shellHook = ''
      figlet "DendrETH"
    '';
  }
