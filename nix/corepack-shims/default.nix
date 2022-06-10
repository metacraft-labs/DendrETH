{
  stdenv,
  nodejs,
}:
stdenv.mkDerivation {
  name = "corepack-shims";
  buildInputs = [nodejs];
  phases = ["installPhase"];
  installPhase = ''
    mkdir -p $out/bin
    corepack enable --install-directory=$out/bin
  '';
}
