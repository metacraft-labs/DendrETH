{
  lib,
  buildGoModule,
}: let
  rootDir = ../../../beacon-light-client/plonky2/gnark_plonky2_verifier;
  inherit (lib) fileset;

  goModFileset = fileset.unions [
    (rootDir + /go.mod)
    (rootDir + /go.sum)
  ];

  srcFiles =
    fileset.fileFilter (
      file: (file.hasExt "go") || (file.hasExt "json")
    )
    rootDir;

  src = fileset.toSource {
    root = rootDir;
    fileset = fileset.union goModFileset srcFiles;
  };
in
  buildGoModule {
    pname = "gnark-plonky2-verifier";
    version = "unstable";
    inherit src;
    vendorHash = "sha256-BebjCtIS5OUfA/R2PNcj926vmMx0G8DsSu+S527GW1k=";
    postInstall = ''
      mv $out/bin/DendrETH $out/bin/gnark-plonky2-verifier
    '';
    meta.mainProgram = "gnark-plonky2-verifier";
  }
