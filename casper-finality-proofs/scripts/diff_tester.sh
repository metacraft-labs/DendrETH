#!/bin/bash

# Circuits
testCircuit='{ "circuit": "WrapperTest", "path": "./src/test_engine/tests/test" }'
hashCircuit='{ "circuit": "WrapperHashTest", "path": "./src/test_engine/tests/hash_test" }'
testLteCircuit='{ "circuit": "WrapperTestLte", "path": "./src/test_engine/tests/test_lte" }'

# Create an array of circuits
circuits=("$testCircuit" "$hashCircuit" "$testLteCircuit")

# Loop over the array of circuits
for data in "${circuits[@]}"
do
    # Extract values from the JSON strings using sed
    circuit=$(echo "$data" | sed -n 's/.*"circuit": "\([^"]*\)".*/\1/p')
    path=$(echo "$data" | sed -n 's#.*"path": "\([^"]*\)".*#\1#p')

    for file in "$path"/*.json
    do
        file_name=$(basename "$file")
        output=$(cargo run  --release --bin differential_tester -- "$circuit" "$file" 2>&1)
        if [[ $output == *"thread 'main' panicked"* ]]; then
            echo "[$circuit] Fail: $file_name"
        else
            echo "[$circuit] Success: $file_name"
        fi
    done
done
