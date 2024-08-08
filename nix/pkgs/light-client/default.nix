{
  lib,
  stdenv,
  gmp,
  autoPatchelfHook,
}:
stdenv.mkDerivation rec {
  pname = "light-client-patched";
  version = "dev";

  src = ../../../vendor/build-artifacts/light_client_cpp;

  nativeBuildInputs = [
    autoPatchelfHook
  ];

  buildInputs = [
    stdenv.cc.cc.lib
    gmp
  ];

  installPhase = ''
    mkdir -p $out/bin
    cp ./light_client $out/bin/light_client
    cp ./light_client.dat $out/bin/light_client.dat
  '';
}
