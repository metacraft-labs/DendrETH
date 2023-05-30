{
  lib,
  pkgs,
  ptau,
  snarkjs,
  ...
}:
with ptau; let
  inherit (pkgs) callPackage fetchFromGitHub runCommand stdenv writeText;
  inherit (builtins) attrNames filter map readDir;
  inherit (lib) pipe hasSuffix removeSuffix;

  llvm = pkgs.llvmPackages_14;
  emscripten = pkgs.metacraft-labs.emscripten;

  mergeListOfSets = list:
    if (builtins.length list < 1)
    then {}
    else
      (
        builtins.head list
      )
      // (
        mergeListOfSets (builtins.tail list)
      );

  circomlib = fetchFromGitHub {
    owner = "iden3";
    repo = "circomlib";
    rev = "cff5ab6288b55ef23602221694a6a38a0239dcc0";
    hash = "sha256-RSpPQxpSp8PvRTZlLIWQmqm3J+Hv+Nx+2cUsz/EcbIQ=";
  };

  circom-pairing = callPackage ./circom-pairing/default.nix {inherit circomlib;};

  transpile_circuit = circuit:
    stdenv.mkDerivation rec {
      name = "${circuit}_cpp";
      src = ../../..;
      buildInputs = [pkgs.metacraft-labs.circom];
      buildPhase = ''
        cd beacon-light-client/circom/scripts/${circuit}
        sed -i 's#../../../vendor/circom-pairing#${circom-pairing}/lib/circom-pairing#' ../../circuits/*.circom
        sed -i 's#../../../node_modules/circomlib#${circomlib}#' ../../circuits/*.circom
        mkdir -p "$out/lib/circom/${circuit}"
        echo "****COMPILING CIRCUIT****"
        circom "${circuit}".circom --O2 --r1cs --sym --c --output "$out/lib/circom/${circuit}"
      '';
    };

  compile_circuit = circuit:
    stdenv.mkDerivation rec {
      name = "${circuit}_build";
      src = circuits."${circuit}_cpp";
      sourceRoot = "${circuit}_cpp/lib/circom/${circuit}/${circuit}_cpp";
      buildInputs = with pkgs; [metacraft-labs.circom snarkjs gnumake nlohmann_json gmp nasm];
      installPhase = ''
        mkdir -p $out/bin
        cp "${circuit}" "${circuit}".dat ../"${circuit}".r1cs $out/bin
      '';
    };

  generate_zkey_0 = circuit: ptau:
    stdenv.mkDerivation rec {
      name = "${circuit}_zkey_0";
      src = circuits."${circuit}_build";
      buildInputs = with pkgs; [nodejs yarn];
      buildPhase = ''
        cd bin
        node --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey new -verbose "${circuit}".r1cs "${ptau}" "${circuit}"_0.zkey -v | tee "${circuit}"_zkey0.out
      '';
      installPhase = ''
        mkdir -p "$out/lib/circom/${circuit}"
        cp "${circuit}"_0.zkey "${circuit}"_zkey0.out "$out/lib/circom/${circuit}"
      '';
    };

  phase_2_ceremony = circuit:
    stdenv.mkDerivation rec {
      name = "${circuit}_phase_2_ceremony";
      src = circuits."${circuit}_zkey_0";
      buildInputs = with pkgs; [nodejs yarn];
      buildPhase = ''
        node ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey contribute -verbose lib/circom/"${circuit}"/"${circuit}"_0.zkey "${circuit}".zkey -n="First phase2 contribution" -e="some random text 5555" | tee contribute.out
      '';
      installPhase = ''
        mkdir -p "$out/lib/circom/${circuit}"
          cp "${circuit}".zkey contribute.out "$out/lib/circom/${circuit}"
      '';
    };

  generate_zkey = circuit: ptau:
    stdenv.mkDerivation rec {
      name = "${circuit}_zkey";
      src = circuits."${circuit}_build";
      buildInputs = with pkgs; [nodejs yarn];
      buildPhase = ''
        cd bin
        node --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc  ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey verify -verbose "${circuit}".r1cs "${ptau}" ${circuits."${circuit}_phase_2_ceremony"}/lib/circom/"${circuit}"/"${circuit}".zkey  | tee "${circuit}"_verify.out
      '';
      installPhase = ''
        mkdir -p "$out/lib/circom/${circuit}"
        cp "${circuit}"_verify.out "$out/lib/circom/${circuit}"
      '';
    };

  generate_vkey = circuit:
    stdenv.mkDerivation rec {
      name = "${circuit}_vkey";
      src = circuits."${circuit}_phase_2_ceremony";
      buildInputs = with pkgs; [nodejs yarn];
      buildPhase = ''
        node ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey export verificationkey "lib/circom/${circuit}/${circuit}".zkey vkey.json -v
      '';
      installPhase = ''
        mkdir -p "$out/lib/circom/${circuit}"
        cp vkey.json "$out/lib/circom/${circuit}"
      '';
    };

  generate_circuit = circuit: ptau: {
    "${circuit}_cpp" = transpile_circuit circuit;
    "${circuit}_build" = compile_circuit circuit;
    # "${circuit}_verify_witness" = verify_witness circuit;
    "${circuit}_zkey_0" = generate_zkey_0 circuit ptau;
    "${circuit}_phase_2_ceremony" = phase_2_ceremony circuit;
    "${circuit}_zkey" = generate_zkey circuit ptau;
    "${circuit}_vkey" = generate_vkey circuit;
    # "${circuit}_generate_proof" = generate_proof circuit;
    # "${circuit}_verify_proof" = verify_proof circuit;
    "${circuit}_full" = stdenv.mkDerivation rec {
      name = "${circuit}_list";
      buildInputs = [
        circuits."${circuit}_cpp"
        circuits."${circuit}_build"
        circuits."${circuit}_zkey_0"
        circuits."${circuit}_phase_2_ceremony"
        circuits."${circuit}_zkey"
        circuits."${circuit}_vkey"
      ];

      dontUnpack = true;
      installPhase = ''
        mkdir -p $out
        touch "$out/${circuit}_list"
      '';
    };
  };

  generate_circuits = circuits:
    lib.trivial.pipe circuits [
      (map (x: generate_circuit x.circuit x.ptau))
      mergeListOfSets
    ];

  circuits = with pkgs;
    generate_circuits [
      {
        circuit = "aggregate_bitmask"; # Either freezes at  Writing points end B1: 4024071/6400950, or it simply takes so long that 3 days aren't enough
        ptau = ptau24;
      }
      {
        circuit = "compress";
        ptau = ptau25;
      }
      {
        circuit = "compute_domain";
        ptau = ptau18;
      }
      {
        circuit = "compute_signing_root";
        ptau = ptau18;
      }
      {
        circuit = "expand_message";
        ptau = ptau21;
      }
      {
        circuit = "hash_to_field";
        ptau = ptau21;
      }
      {
        circuit = "hash_tree_root";
        ptau = ptau21;
      }
      {
        circuit = "hash_tree_root_beacon_header";
        ptau = ptau20;
      }
      {
        circuit = "is_supermajority";
        ptau = ptau10;
      }
      {
        circuit = "is_valid_merkle_branch";
        ptau = ptau20;
      }
      {
        circuit = "light_client"; # Not enough memory, circom crashes
        ptau = ptau20;
      }
      {
        circuit = "light_client_recursive"; # Typing error in .circom code
        ptau = ptau20;
      }
    ];
in
  circuits
