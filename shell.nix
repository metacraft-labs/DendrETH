{ pkgs }: with pkgs;
let
  nodejs = nodejs-16_x;
in
mkShell {
  buildInputs = [
    # For priting the direnv banner
    figlet

    # For an easy way to launch all required blockchain simulations
    # and tailed log files
    tmux
    tmuxinator

    # Node.js dev environment for unit tests
    nodejs
    (yarn.override { inherit nodejs; })
  ];

  shellHook = ''
    figlet "DendrETH"
  '';
}
