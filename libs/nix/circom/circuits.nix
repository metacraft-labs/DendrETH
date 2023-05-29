{
  lib,
  pkgs,
  ptau,
  snarkjs,
  ...
}:
with ptau; let
  inherit (pkgs) callPackage fetchFromGitHub runCommand stdenv;
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
      name = "${circuit}_circuit";
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

  compile_circuit = circuit: ptau:
    stdenv.mkDerivation rec {
      name = "${circuit}";
      src = circuits."${circuit}_circuit";
      sourceRoot = "${circuit}_circuit/lib/circom/${circuit}/${circuit}_cpp";
      buildInputs = with pkgs; [metacraft-labs.circom snarkjs gnumake nodejs nlohmann_json gmp nasm yarn];
      preBuild = ''
        echo "****COMPILING C++ WITNESS GENERATION CODE****";
      '';
      postBuild = ''
        # echo "****VERIFYING WITNESS****"
        # ./"${circuit}" ../../../scripts/"${circuit}"/input.json witness.wtns
        # node ${snarkjs}/lib/node_modules/snarkjs/cli.js wej witness.wtns witness.json

        echo "****GENERATING ZKEY 0****"
        node --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey new ../"${circuit}".r1cs "${ptau}" "${circuit}"_0.zkey -v | tee "${circuit}"_zkey0.out

        echo "****CONTRIBUTE TO PHASE 2 CEREMONY****"
        node ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey contribute -verbose "${circuit}"_0.zkey "${circuit}".zkey -n="First phase2 contribution" -e="some random text 5555" > contribute.out

        echo "****VERIFYING FINAL ZKEY****"
        node --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc  ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey verify -verbose ../"${circuit}".r1cs "${ptau}" "${circuit}".zkey  > "${circuit}"_verify.out

        echo "****EXPORTING VKEY****"
        node ${snarkjs}/lib/node_modules/snarkjs/cli.js zkey export verificationkey "${circuit}".zkey vkey.json -v

        # echo "****GENERATING PROOF FOR SAMPLE INPUT****"
        # ${pkgs.metacraft-labs.rapidsnark}/bin/prover "${circuit}".zkey witness.wtns proof.json public.json > proof.out

        # echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
        # node ${snarkjs}/lib/node_modules/snarkjs/cli.js groth16 verify vkey.json public.json proof.json -v

      '';
      installPhase = ''
        mkdir -p $out/bin
        cp "${circuit}" "${circuit}".dat "${circuit}"_0.zkey "${circuit}".zkey vkey.json ../"${circuit}".r1cs $out/bin
      '';
    };

  generate_circuit = circuit: ptau: {
    "${circuit}_circuit" = transpile_circuit circuit;
    "${circuit}" = compile_circuit circuit ptau;
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
