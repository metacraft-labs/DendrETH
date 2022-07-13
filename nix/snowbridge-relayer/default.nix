{pkgs}:
with pkgs;
  buildGoModule rec {
    pname = "snowbridge-relayer";
    version = "0.0.1";
    src = fetchFromGitHub {
      owner = "snowfork";
      repo = "snowbridge";
      sha256 = "sha256-bIaPwxcWaTO60L4Qur5gmqEmVHfCtaS5c6rVqCL1dpg=";
      rev = "a65d9118dd4b2277eb7a8513c6f9d7273f277fc2";
    };

    vendorSha256 = "sha256-SmZ8tJHsqlnRBhDv6wXoDfDenUso7j1RvszlYYoqK+k=";

    sourceRoot = "source/relayer";

    CGO_ENABLED = 0;

    nativeBuildInputs = with pkgs; [
      stdenv.cc.cc
      go
      mage
      # revive
    ];
    buildInputs = with pkgs; [
      zlib
      jq
      # abigen
    ];

    doCheck = false;

    buildPhase = ''
      runHook preBuild
      export HOME=$(mktemp -d)
      mage Build
      runHook postBuild
    '';

    checkPhase = ''
      runHook preCheck
      mage Test
      runHook postCheck
    '';

    installPhase = ''
      runHook preInstall
      install -Dt $out/bin build/snowbridge-relay
      runHook postInstall
    '';

    meta = with lib; {
      homepage = "https://github.com/Snowfork/snowbridge";
      platforms = platforms.linux;
    };
  }
