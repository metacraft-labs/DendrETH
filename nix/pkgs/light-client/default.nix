{
  stdenv,
  autoPatchelfHook,
  fetchgit,
  nasm,
  gmp,
  nlohmann_json,
  lib,
}:
stdenv.mkDerivation rec {
  pname = "light-client";
  version = "dev-23-03-23";

  src = fetchgit {
    url = "https://github.com/metacraft-labs/DendrETH-build-artifacts";
    rev = "46ef993a007175640b4ff5dd8855c12a26f57d7a";
    hash = "sha256-bbw/10O4FOARzR7+7kOP8X3yRM02UxfVMEAwQ6cbVMc=";
    fetchLFS = true;
  };

  patchPhase = ''
    cd light_client_cpp;
    rm -rf *.o light_client
  '';

  nativeBuildInputs = [
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
