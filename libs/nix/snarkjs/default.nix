{
  buildNpmPackage,
  lib,
  fetchFromGitHub,
}:
buildNpmPackage rec {
  pname = "snarkjs";
  version = "0.6.11";

  npmDepsHash = "sha256-H9F9AmR6WmBj+Q/6xTel2pVvJhx5etZbNacD8IBLWpQ=";

  src = fetchFromGitHub {
    owner = "iden3";
    repo = "snarkjs";
    rev = "v${version}";
    hash = "sha256-uYDuNaE+ymgrVbpzfWeCpYeQwd5Sp5mIq5e1Mk4zBpw=";
  };

  meta = with lib; {
    description = "ZkSNARK implementation in JavaScript & WASM";
    homepage = "https://github.com/iden3/snarkjs";
    license = licenses.gpl3Only;
    maintainers = with maintainers; [];
  };
}
