finalNixpkgs: prevNixpkgs: let
in {
  cargo-contract = prevNixpkgs.callPackage ./nix/cargo-contract {};
  dylint = prevNixpkgs.callPackage ./nix/dylint {};
  rust-clippy = prevNixpkgs.callPackage ./nix/rust-clippy {};
}
