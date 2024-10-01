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
    inherit (self'.packages) gnark-plonky2-verifier;

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
            RUST_BACKTRACE=1 ${getExe pkg} --serialized-circuits-dir ./serialized_circuits
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
        BIN=$out/bin/${executable.meta.mainProgram}

        # Some circuit executables (like the pubkey-commitment-mapper) don't
        # have pre-built circuit data and instead build their circuits on
        # startup. In such case, instead of pointing the
        # `--serialized-circuits-dir` to the Nix package containing the
        # pre-built circuits, point to ./serialized_circuits (relative to the
        # working directory of the circuit-executable). That way, when the
        # executable starts, it will create this directory and store its files
        # there.
        CIRCUIT_DIR=${
          if circuit-data == []
          then "./serialized_circuits"
          else "$out/data/serialized_circuits"
        }
        mkdir -p $CIRCUIT_DIR $out/bin

        cat << EOF > $BIN
        #!${pkgs.runtimeShell}
        set -x
        RUST_BACKTRACE=1 ${getExe executable} --serialized-circuits-dir $CIRCUIT_DIR --proof-storage-cfg ${proof-storage-config} "\$@"
        EOF
        chmod +x $BIN

        ${concatMapStringsSep "\n" (data: "ln -fs ${data}/* $CIRCUIT_DIR") circuit-data}
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
        range = range 0 32;
      };
      deposit-accumulator-balance-aggregator-final-layer = {
        binaryName = "deposit_accumulator_balance_aggregator_final_layer";
        linkCircuitData = circuits:
          circuits.deposit-accumulator-balance-aggregator-diva.circuit-data."32"
          ++ circuits.pubkey-commitment-mapper.circuit-data."32"
          ++ circuits.commitment-mapper.circuit-data."24"
          ++ circuits.commitment-mapper.circuit-data."40";
      };
      pubkey-commitment-mapper = {
        binaryName = "pubkey_commitment_mapper";
      };
      final-layer = {
        binaryName = "final_layer";
        linkCircuitData = circuits:
          circuits.balance-verifier.circuit-data."37"
          ++ circuits.commitment-mapper.circuit-data."40";
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
              then let
                circuit-data-drv = buildCircuit circuit-builder range;
                levels = map toString range;
              in
                lib.genAttrs levels (l: [circuit-data-drv."level_${toString l}"])
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
          in
            {
              inherit binary;
            }
            // lib.optionalAttrs (builderName != null) {inherit circuit-builder circuit-data;}
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
            if linkCircuitData == null
            then circuit-info
            else circuit-info // {circuit-data."0" = linked-circuit-data;}
        )
        mapping;

      linked-circuits = linkCircuitData mapping;

      circuits-with-levels =
        mapAttrs (
          name: value @ {
            binary,
            circuit-data ? {},
            levels ? {},
            ...
          }:
            value
            // {
              levels =
                {
                  all = packageCircuitExecuable "all" binary (
                    lib.flatten (builtins.attrValues
                      circuit-data)
                  );
                }
                // (lib.mapAttrs (level: data: packageCircuitExecuable level binary data) circuit-data);
            }
        )
        linked-circuits;
    in
      circuits-with-levels;

    circuit-executables = packageAllCircuitExecutables mapping;

    circuit-executable-images = lib.pipe circuit-executables [
      builtins.attrValues
      (map (executable: lib.mapAttrsToList (name: level: level.image) executable.levels))
      lib.flatten
    ];

    gnark-plonky2-verifier-image = buildToolImage {
      mainPackage = gnark-plonky2-verifier;
    };

    copy-images-to-docker-daemon = writeScriptBin "circuit-executable-images" (
      concatMapStringsSep "\n" (level: getExe level.copyToDockerDaemon) (
        circuit-executable-images
        ++ [
          gnark-plonky2-verifier-image
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

    proof-storage-config = pkgs.writeTextFile {
      name = "proof-storage-config";
      text = builtins.toJSON (import ./proof-storage-config.nix);
    };
  in {
    legacyPackages = {
      inherit
        proof-storage-config
        mapping
        circuit-executables
        circuit-executable-images
        copy-images-to-docker-daemon
        # misc-images
        ;

      wrapper = installRustBinary "wrapper" "wrapper";
    };
    packages = {
      # inherit all-circuit-executables balance-verifier-circuit-builder input-fetchers;
    };
  };
}
