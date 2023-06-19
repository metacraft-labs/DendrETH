{
  pkgs,
  lib,
  stdenv,
  autoPatchelfHook,
  fetchgit,
}:
stdenv.mkDerivation rec {
  pname = "light-client";
  version = "dev-23-03-23";

  src = fetchgit {
    url = "https://github.com/metacraft-labs/DendrETH-build-artifacts";
    rev = "24a97aae330fc12007fe51d4918293f5fdc2e62b";
    hash = "sha256-Yn8hsvdxb5H4gSy3IsXi9inu0m3WdijK1CZep6HNDbU=";
    fetchLFS = true;
  };

  # sourceRoot = "source/light_client_cpp";

  patchPhase = ''
    cd light_client_cpp;
    rm -rf *.o light_client
  '';

  nativeBuildInputs = with pkgs; [
    gcc
    gnumake
    nasm
    autoPatchelfHook
    gmp
    nlohmann_json
  ];

  installPhase = ''
    mkdir -p $out/bin
    cp ./light_client $out/bin/light_client
    cp ./light_client.dat $out/bin/light_client.dat
  '';
}
