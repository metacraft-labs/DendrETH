{
  buildNpmPackage,
  lib,
  fetchFromGitHub,
  circomlib,
}:
buildNpmPackage rec {
  pname = "circom-pairing";
  version = "unstable-2023-01-09";

  src = fetchFromGitHub {
    owner = "metacraft-labs";
    repo = "circom-pairing";
    rev = "62c18134f6b1fcc241cbdd67586f92f34e7aca16";
    hash = "sha256-h3JZ9mH0ZJcScMrVeb3LUPhvvSvryFRvEoTZRs0qzcE=";
    fetchSubmodules = true;
  };

  dontNpmBuild = true;

  npmDepsHash = "sha256-yQDLB7ecv9YRhjeEvsKnYnTpeuQMKWtG70ujTDdkBE8=";

  postPatch = ''
    cp ${./package-lock.json} package-lock.json
  '';

  installPhase = ''
    find . -name "*.circom" -exec sed -i 's#../../../node_modules/circomlib#${circomlib}#' {} +


    mkdir -p "$out/lib/circom-pairing"
    cp -r . "$out/lib/circom-pairing"
  '';

  meta = with lib; {
    description = "";
    homepage = "https://github.com/metacraft-labs/circom-pairing";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [];
  };
}
