{...}: {
  perSystem = {pkgs, ...}: {
    packages = {
      light-client = pkgs.callPackage ./light-client {};
    };
  };
}
