{inputs, ...}: {
  perSystem = {
    lib,
    self',
    inputs',
    pkgs,
    ...
  }: let
    inherit (lib) getExe concatMapStringsSep range assertMsg hasSuffix removeSuffix mapAttrs;
    inherit (pkgs) callPackage runCommand runCommandLocal writeScriptBin;
    inherit (inputs'.mcl-blockchain.legacyPackages) nix2container;

    all-circuit-executables = self'.packages.circuit-executables;

    buildToolImage = {
      mainPackage,
      name ? mainPackage.pname,
    }:
      nix2container.buildImage {
        inherit name;
        tag = mainPackage.version or "latest";
        copyToRoot = pkgs.buildEnv {
          name = "root";
          paths = [mainPackage];
          pathsToLink = ["/bin"];
        };
        config = {
          entrypoint = ["/bin/${mainPackage.meta.mainProgram}"];
          workingdir = "/bin";
        };
      };

    nodejs = pkgs.nodejs_21;

    buildCircuit = pkg: levels: let
      outputs = ["out"] ++ (map (i: "level_${toString i}") levels);
    in
      assert assertMsg (hasSuffix "-builder" pkg.name)
      "The package name must end with '-builder', but got: ${pkg.name}";
        runCommand "${(removeSuffix "-builder" pkg.name)}-data" {inherit outputs;} (
          ''
            ${getExe pkg}
          ''
          + lib.concatMapStringsSep "\n" (i: ''
            mkdir $level_${toString i}
            mv serialized_circuits/*_${toString i}.plonky2_{targets,circuit} $level_${toString i}
            mkdir -p $out
            for f in $level_${toString i}/*; do
              ln -s $f $out/$(basename $f)
            done
          '')
          levels
        );

    installRustBinary = pname: exeName:
      runCommandLocal pname {meta.mainProgram = exeName;} ''
        install -Dm755 ${all-circuit-executables}/bin/${exeName} -t $out/bin/
      '';

    packageCircuitExecuable = level: executable: circuit-data: let
      name = "${executable.name}-${level}";
      pkg = runCommand name {inherit (executable) meta;} ''
        mkdir -p $out/bin/serialized_circuits
        install -Dm755 ${getExe executable} -t $out/bin/
        ${concatMapStringsSep "\n" (data: "ln -s ${data}/* $out/bin/serialized_circuits") circuit-data}
      '';
      image = buildToolImage {
        inherit name;
        mainPackage = pkg;
      };
    in {
      inherit pkg image;
    };

    mapping = {
      balance-verifier = {
        binaryName = "balance_verification";
        builderName = "balance_verification_circuit_data_generation";
        range = range 0 37;
        argsBuilder = level: let
          mkLevel = level: toString level;
        in
          if level == 0
          then [(mkLevel level)]
          else [
            (mkLevel level)
            (mkLevel (level - 1))
          ];
      };
      commitment-mapper = {
        binaryName = "commitment_mapper";
        builderName = "commitment_mapper_circuit_data_generation";
        range = range 0 40;
      };
      deposit-accumulator-balance-aggregator-diva = {
        binaryName = "deposit_accumulator_balance_aggregator_diva";
        builderName = "deposit_accumulator_balance_aggregator_diva_circuit_data_generation";
      };
      deposit-accumulator-balance-aggregator-final-layer = {
        binaryName = "deposit_accumulator_balance_aggregator_final_layer";
      };
      pubkey-commitment-mapper = {
        binaryName = "pubkey_commitment_mapper";
      };
      final-layer = {
        binaryName = "final_layer";
        linkCircuitData = circuits:
          circuits.balance-verifier.circuit-data."37" ++ circuits.commitment-mapper.circuit-data."0";
      };
    };

    packageAllCircuitExecutables = mapping: let
      buildCircuitData = mapping:
        mapAttrs (
          name: {
            binaryName,
            builderName ? null,
            range ? null,
            argsBuilder ? null,
            ...
          }: let
            binary = installRustBinary name binaryName;
            circuit-builder = installRustBinary "${name}-builder" builderName;
            circuit-data =
              if builderName == null
              then {}
              else if range == null
              then {"0" = [(buildCircuit circuit-builder [0])];}
              else if argsBuilder == null
              then {"0" = [(buildCircuit circuit-builder range)];}
              else let
                circuit-data = buildCircuit circuit-builder range;
              in
                lib.pipe range [
                  (map (
                    level: let
                      args = argsBuilder level;
                    in {
                      name = toString level;
                      value = map (l: circuit-data."level_${toString l}") args;
                    }
                  ))
                  lib.listToAttrs
                ];
            levels = lib.mapAttrs (level: data: packageCircuitExecuable level binary data) circuit-data;
          in
            {
              inherit binary;
              levels."0" = packageCircuitExecuable "0" binary [];
            }
            // lib.optionalAttrs (builderName != null) {inherit circuit-builder circuit-data levels;}
        )
        mapping;

      all-deps-free-circuit-data = buildCircuitData mapping;

      linkCircuitData = mapping:
        mapAttrs (
          name: {linkCircuitData ? null, ...}: let
            circuit-info = all-deps-free-circuit-data.${name};
            linked-circuit-data =
              if linkCircuitData == null
              then circuit-info.circuit-data
              else linkCircuitData all-deps-free-circuit-data;
          in
            circuit-info // {circuit-data = linked-circuit-data;}
        )
        mapping;
    in
      linkCircuitData mapping;

    circuit-executables = packageAllCircuitExecutables mapping;

    circuit-executable-images = lib.pipe circuit-executables [
      builtins.attrValues
      (map (executable: lib.mapAttrsToList (name: level: level.image) executable.levels))
      lib.flatten
    ];

    copy-images-to-docker-daemon = writeScriptBin "circuit-executable-images" (
      concatMapStringsSep "\n" (level: getExe level.copyToDockerDaemon) (
        circuit-executable-images
        ++ [
        ]
      )
    );

    input-fetchers = callPackage ../pkgs/input-fetchers {inherit nodejs;};
    input-fetchers-image = buildToolImage {
      name = "input-fetchers";
      mainPackage = input-fetchers;
    };
    misc-images = writeScriptBin "misc-images" (
      concatMapStringsSep "\n" (image: getExe image.copyToDockerDaemon) [input-fetchers-image]
    );
  in {
    legacyPackages = {
      inherit
        mapping
        circuit-executables
        circuit-executable-images
        copy-images-to-docker-daemon
        # misc-images
        ;
    };
    packages = {
      # inherit all-circuit-executables balance-verifier-circuit-builder input-fetchers;
    };
  };
}
