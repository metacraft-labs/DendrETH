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
    inherit (pkgs) callPackage rust-bin runCommandLocal writeScript;
    inherit (lib) getExe;

    rust-nightly = rust-bin.nightly."2023-06-12".default;

    craneLib = (crane.mkLib pkgs).overrideToolchain rust-nightly;

    circuits-executable = exeName: let
      all = callPackage ../libs/nix/circuits_executables {
        inherit craneLib;
      };
    in
      runCommandLocal exeName {
        meta.programName = exeName;
      } ''
        install -Dm755 ${all}/bin/${exeName} -t $out/bin
      '';

    balance-verifier-circuit-builder = circuits-executable "balance_verification_circuit_data_generation";
    balance-verifier = circuits-executable "balance_verification";
    commitment-mapper = circuits-executable "commitment_mapper";

    balance-verification-circuit = level:
      runCommandLocal "balance-verification-circuit-per-level-${level}" {} ''
        ${getExe balance-verifier-circuit-builder} ${lib.optionalString (level != "all") "--level ${level}"}
        mkdir -p $out/bin
        mv *.plonky2_targets *.plonky2_circuit $out/bin
      '';

    allLevels = builtins.map builtins.toString (lib.lists.range 0 38);
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

    balance-verifier-circuit-per-level-docker = lib.genAttrs allLevels buildImage;

    balance-verifier-all-images =
      writeScript "balance-verifier-all-images"
      (
        lib.concatMapStringsSep
        "\n"
        (level: getExe (buildImage level).copyToDockerDaemon)
        allLevels
      );
  in {
    legacyPackages = {
      inherit balance-verifier-circuit-per-level balance-verifier-circuit-per-level-docker;
      inherit balance-verifier commitment-mapper balance-verifier-all-images;
    };
  };
}
