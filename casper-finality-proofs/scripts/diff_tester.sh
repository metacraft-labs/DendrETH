#!/bin/bash

# Color for printing
BOLD_BLUE='\033[1;34m'
BOLD_RED='\033[1;31m'
BOLD_GREEN='\033[1;32m'
YELLOW='\033[1;33m'
RED_BG='\033[41m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Array to store failed tests and their files
declare -a failed_tests
declare -a failed_tests_details

process_json() {
    local json_file="$1"

    circuit=$(sed -n 's/.*"circuit": "\([^"]*\)".*/\1/p' "$json_file")
    path=$(sed -n 's#.*"path": "\([^"]*\)".*#\1#p' "$json_file")
    ref=$(sed -n 's#.*"ref": "\([^"]*\)".*#\1#p' "$json_file")

    printf "\n%s %b\n" "Running circuit:" "${BOLD_BLUE}$circuit${NC}"

    for file in "$path"/*.json; do
        file_name=$(basename "$file")
        cargo_output=$(cargo run --release --bin differential_tester -- "$circuit" "$file" 2>&1 | awk '/thread .*main.* panicked/,/note:/ {if (!/note:/) print}' | tail -n +2)
        python_output=$(python3 "$ref" "$file" 2>&1)
        python_exit_code=$?

        # Either cargo_output or python_exit_code is not 0 -> test failed
        if [[ -n "$cargo_output" || $python_exit_code -ne 0 ]]; then
            # If test is not supposed to fail but one of the scripts fails -> test failed
            if [[ "$file_name" != *_fail.json && ((-n "$cargo_output" && $python_exit_code -eq 0) || (-z "$cargo_output" && $python_exit_code -ne 0)) ]]; then

                echo "-> ${RED_BG}$file_name${NC}"
                failed_tests+=("-> ${BOLD_BLUE}[$circuit]${NC} ${YELLOW}$file_name${NC}: $cargo_output$python_output")
                failed_tests_details+=("$file")
            else
                echo "-> ${GREEN}$file_name${NC}"
            fi
        elif [[ "$file_name" == *_fail.json ]]; then
            echo "-> ${RED_BG}$file_name${NC}"
            failed_tests+=("-> ${BOLD_BLUE}[$circuit]${NC} ${YELLOW}$file_name${NC}: Error: Test is supposed to fail but it passed.")
            failed_tests_details+=("$file")
        else
            echo "-> ${GREEN}$file_name${NC}"
        fi
    done
}

if [ $# -eq 1 ]; then
    filepath="./scripts/differential_setup/$1.json"
    if [ -f "$filepath" ]; then
        process_json "$filepath"
    else
        echo "${BOLD_RED}Error: File not found! $filepath${NC}"
        exit 1
    fi
else
    for filepath in ./scripts/differential_setup/*.json; do
        if [ -f "$filepath" ]; then
            process_json "$filepath"
        fi
    done
fi

if [ ${#failed_tests[@]} -eq 0 ]; then
    printf "\n%b" "${BOLD_GREEN}All tests passed!${NC}"
else
    # Print failed tests and their errors
    printf "\n%b" "${BOLD_RED}Failed tests:${NC}"
    for ((i = 0; i < ${#failed_tests[@]}; i++)); do
        printf "\n%b\n" "${failed_tests[$i]}"
        cat "${failed_tests_details[$i]}"
    done
fi
