#!/bin/bash

# ANSI escape codes for coloring output
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get all PIDs
all_pids=$(ps -e -o pid=)

# Initialize counts
all_count=0
nix_count=0

# Loop over all PIDs
for pid in $all_pids; do
    # Skip if the process is this script itself
    if [ $pid -eq $$ ]; then
        continue
    fi

    # Get full path of process
    full_path=$(readlink -f /proc/$pid/exe 2>/dev/null)

    # Only count processes with a path
    if [[ -n $full_path ]]; then
        # Increment total count
        ((all_count++))

        # Check if it starts with /nix
        if [[ $full_path == /nix* ]]; then
            # Increment nix count
            ((nix_count++))
            # Print in red
            echo -e "${RED}Nix process found: $full_path${NC}"
        else
            # Print in normal color
            echo "Process: $full_path"
        fi
    fi
done

# Calculate the ratio
if [ "$all_count" -gt 0 ]; then
    ratio=$(echo "scale=2; $nix_count / $all_count" | bc)
else
    ratio=0
fi

echo "Number of /nix processes: $nix_count"
echo "Ratio of /nix processes to all processes: $ratio"
