FROM nixos/nix

RUN nix-channel --update
RUN nix-env -iA nixpkgs.git

RUN git clone https://github.com/metacraft-labs/DendrETH.git -b diva-deployment /app

WORKDIR /app
RUN nix develop --accept-flake-config --extra-experimental-features 'nix-command flakes' -c yarn
