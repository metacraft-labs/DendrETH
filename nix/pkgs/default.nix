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
      circuit-executables = callPackage ../pkgs/circuits_executables {inherit craneLib;};
    };
  };
}
