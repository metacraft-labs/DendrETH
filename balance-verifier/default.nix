{inputs, ...}: {
  perSystem = {
    lib,
    inputs',
    system,
    ...
  }: let
    inherit (inputs.mcl-blockchain.inputs) crane;
    inherit (inputs'.mcl-blockchain.legacyPackages) nix2container pkgs-with-rust-overlay;
    pkgs = pkgs-with-rust-overlay;
    inherit (pkgs) callPackage rust-bin runCommandLocal writeScriptBin;
    inherit (lib) getExe;

    nodejs = pkgs.nodejs_21;

    rust-nightly = rust-bin.nightly."2024-03-28".default;

    craneLib = (crane.mkLib pkgs).overrideToolchain rust-nightly;

    all-circuit-executables = callPackage ../libs/nix/circuits_executables {
      inherit craneLib;
    };

    circuit-executable = exeName:
      runCommandLocal exeName {
        meta.programName = exeName;
      } ''
        install -Dm755 ${all-circuit-executables}/bin/${exeName} -t $out/bin
      '';

    balance-verifier-circuit-builder = circuit-executable "balance_verification_circuit_data_generation";
    balance-verifier = circuit-executable "balance_verification";
    commitment-mapper = circuit-executable "commitment_mapper";
    commitment-mapper-builder = circuit-executable "commitment_mapper_circuit_data_generation";
    final-layer = circuit-executable "final_layer";

    balance-verification-circuit = level:
      runCommandLocal "balance-verification-circuit-per-level-${level}" {} ''
        ${getExe balance-verifier-circuit-builder} ${lib.optionalString (level != "all") "--level ${level}"}
        mkdir -p $out/bin
        mv *.plonky2_targets *.plonky2_circuit $out/bin
      '';

    commitment-mapper-data = runCommandLocal "commitment-mapper-data" {} ''
      ${getExe commitment-mapper-builder}
      mkdir -p $out/bin
      mv *.plonky2_targets *.plonky2_circuit $out/bin
    '';

    allLevels = builtins.map builtins.toString (lib.lists.range 0 37);
    balance-verifier-circuit-per-level = lib.genAttrs (allLevels ++ ["all"]) balance-verification-circuit;

    buildImage = level: let
      levelBefore = toString (lib.toInt level - 1);
    in
      nix2container.buildImage {
        name = "balance-verifier-for-level-${level}";
        tag = "latest";
        copyToRoot = pkgs.buildEnv {
          name = "root";
          paths = [balance-verifier balance-verifier-circuit-per-level."${level}" pkgs.bash pkgs.coreutils (lib.optionalString (level != "0") balance-verifier-circuit-per-level."${levelBefore}")];
          pathsToLink = ["/bin"];
        };
        config = {
          entrypoint = ["/bin/${balance-verifier.meta.programName}"];
          workingdir = "/bin";
        };
      };

    buildToolImage = tool:
      nix2container.buildImage {
        name = "${builtins.replaceStrings ["-"] ["_"] tool.name}";
        tag = "latest";
        copyToRoot = pkgs.buildEnv {
          name = "root";
          paths = [tool];
          pathsToLink = ["/bin"];
        };
        config = {
          workingdir = "/bin";
        };
      };

    commitment-mapper-image = nix2container.buildImage {
      name = "commitment_mapper";
      tag = "latest";
      copyToRoot = pkgs.buildEnv {
        name = "root";
        paths = [commitment-mapper commitment-mapper-data];
        pathsToLink = ["/bin"];
      };
      config = {
        workingdir = "/bin";
      };
    };

    final-layer-image = nix2container.buildImage {
      name = "final-layer";
      tag = "latest";
      copyToRoot = pkgs.buildEnv {
        name = "root";
        paths = [final-layer];
        pathsToLink = ["/bin"];
      };
      config = {
        entrypoint = ["/bin/${final-layer.meta.programName}"];
        workingdir = "/bin";
      };
    };

    balance-verifier-circuit-per-level-docker = lib.genAttrs allLevels buildImage;

    balance-verifier-all-images =
      writeScriptBin "balance-verifier-all-images"
      (
        lib.concatMapStringsSep
        "\n"
        (level: getExe (buildImage level).copyToDockerDaemon)
        allLevels
      );

    input-fetchers = callPackage ../libs/nix/input-fetchers {inherit nodejs;};
    misc-images =
      writeScriptBin "misc-images"
      (
        lib.concatMapStringsSep
        "\n"
        (image: getExe image.copyToDockerDaemon)
        ((map buildToolImage [input-fetchers])
          ++ [commitment-mapper-image])
      );
  in {
    legacyPackages = {
      inherit balance-verifier-circuit-per-level balance-verifier-circuit-per-level-docker commitment-mapper-data;
      inherit balance-verifier commitment-mapper balance-verifier-all-images final-layer final-layer-image commitment-mapper-image;
      inherit misc-images;
    };
    packages = {
      inherit all-circuit-executables balance-verifier-circuit-builder input-fetchers;
    };
  };
}
