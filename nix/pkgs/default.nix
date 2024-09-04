{...}: {
  perSystem = {
    pkgs,
    self',
    ...
  }: let
    inherit (pkgs) callPackage;
    inherit (self'.legacyPackages.rustToolchain) craneLib;
  in {
    packages = {
      light-client = callPackage ./light-client {};
      input-fetchers = callPackage ./input-fetchers {};
      circuit-executables = callPackage ../pkgs/circuits_executables {inherit craneLib;};
      gnark-plonky2-verifier = callPackage ./gnark-plonky2-verifier {};
    };
  };
}
