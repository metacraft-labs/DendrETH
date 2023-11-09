#!/bin/bash

# Function to process each directory
process_directory() {
    local corpus_dir="$1"
    local hex="$2"
    local count="$3"

    filename="0x${hex:2:7}_${count}"
    echo "{ \"seed\": \"${hex}\", \"count\": ${count} }" >"${corpus_dir}/${filename}"
}

# Main script
main() {
    type="$1"
    script_dir=$(dirname "$(realpath "$0")")
    project_root=$(realpath "$script_dir"/../../../)

    corpus_dir="$project_root/casper-finality-proofs/fuzz/corpus/compute_shuffled_index_$type"
    seed_dir="$project_root/vendor/consensus-spec-tests/tests/minimal/phase0/shuffling/core/shuffle"

    # Check if the directory exists, if not, create it
    if [ ! -d "$corpus_dir" ]; then
        echo "Directory doesn't exist. Creating directory..."
        mkdir -p "$corpus_dir"
    elif [ ! -d "$corpus_dir" ] || [ -z "$(ls -A "$corpus_dir")" ]; then
        echo "Directory is empty or doesn't exist. Creating files..."
    else
        echo "Directory is not empty. Exiting..."
        exit 0
    fi

    echo "The full path of the directory is: $(realpath "$corpus_dir")"

    # Loop through each folder
    for folder in "$seed_dir"/*; do
        if [ -d "$folder" ] && [[ "$(basename "$folder")" == shuffle_* ]]; then
            hex=$(echo "$(basename "$folder")" | cut -d'_' -f 2)
            count=$(echo "$(basename "$folder")" | cut -d'_' -f 3)
            case $count in
            1 | 2 | 3 | 5)
                process_directory "$corpus_dir" "$hex" "$count"
                ;;
            *)
                echo "Skipping count $count for folder $folder."
                ;;
            esac
        fi
    done
}

# Run the script
main "$1"
