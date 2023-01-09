#!/usr/bin/env bash

PHASE1=../../../../../../../powers_of_tau/pot28_final.ptau
CIRCUIT_NAME=is_valid_merkle_branch
BUILD_DIR=../../build/"$CIRCUIT_NAME"

if [ -f "$PHASE1" ]; then
    echo "Found Phase 1 ptau file"
else
    echo "No Phase 1 ptau file found. Exiting..."
    exit 1
fi

if [ ! -d "$BUILD_DIR" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "$BUILD_DIR"
fi

echo $PWD

echo "****COMPILING CIRCUIT****"
start=`date +%s`
#circom "$CIRCUIT_NAME".circom --O0 --c --output "$BUILD_DIR"
circom "$CIRCUIT_NAME".circom --O2 --r1cs --sym --c --output "$BUILD_DIR"
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****COMPILING C++ WITNESS GENERATION CODE****"
start=`date +%s`
cd "$BUILD_DIR"/"$CIRCUIT_NAME"_cpp
make
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****VERIFYING WITNESS****"
start=`date +%s`
./"$CIRCUIT_NAME" ../../../scripts/"$CIRCUIT_NAME"/input.json ../witness.wtns
end=`date +%s`
echo "DONE ($((end-start))s)"

cd ..
# yarn snarkjs wej $PWD/witness.wtns witness.json

echo "****GENERATING ZKEY 0****"
start=`date +%s`
node --max-old-space-size=2048000 --initial-old-space-size=2048000 --max-semi-space-size=1024 --initial-heap-size=2048000 ../../../../node_modules/snarkjs/cli.js zkey new "$CIRCUIT_NAME".r1cs "$PHASE1" "$CIRCUIT_NAME"_0.zkey -v > zkey0.out
end=`date +%s`
echo "DONE ($((end-start))s)"

# echo "****CONTRIBUTE TO PHASE 2 CEREMONY****"
# start=`date +%s`
# node ../../../../node_modules/snarkjs/cli.js zkey contribute -verbose "$CIRCUIT_NAME"_0.zkey "$CIRCUIT_NAME".zkey -n="First phase2 contribution" -e="some random text 5555" > contribute.out
# end=`date +%s`
# echo "DONE ($((end-start))s)"

# echo "****VERIFYING FINAL ZKEY****"
# start=`date +%s`
# node --trace-gc --trace-gc-ignore-scavenger --max-old-space-size=2048000 --initial-old-space-size=2048000 --no-global-gc-scheduling --no-incremental-marking --max-semi-space-size=1024 --initial-heap-size=2048000 --expose-gc ../../../../node_modules/snarkjs/cli.js zkey verify -verbose "$CIRCUIT_NAME".r1cs "$PHASE1" "$CIRCUIT_NAME".zkey > verify.out
# end=`date +%s`
# echo "DONE ($((end-start))s)"

echo "****EXPORTING VKEY****"
start=`date +%s`
node ../../../../node_modules/snarkjs/cli.js zkey export verificationkey "$CIRCUIT_NAME"_0.zkey vkey.json -v
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****GENERATING PROOF FOR SAMPLE INPUT****"
start=`date +%s`
../../../../vendor/rapidsnark/build/prover "$CIRCUIT_NAME"_0.zkey witness.wtns proof.json public.json > proof.out
end=`date +%s`
echo "DONE ($((end-start))s)"

echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
start=`date +%s`
node ../../../../node_modules/snarkjs/cli.js groth16 verify vkey.json public.json proof.json -v
end=`date +%s`
echo "DONE ($((end-start))s)"
