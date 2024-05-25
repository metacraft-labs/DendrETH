#!/usr/bin/env sh

# defaults
REDIS_ADDRESS=redis://localhost:6379

while [[ $# -gt 0 ]]; do
    case $1 in
        --redis-address)
            REDIS_ADDRESS="$2"
            shift
            shift
            ;;
        --protocol)
            PROTOCOL="$2"
            shift
            shift
            ;;
        --proof-storage-dir)
            PROOF_STORAGE_DIR="$2"
            shift
            shift
            ;;
        -*|--*)
            echo "ERROR: Unknown option $1"
            exit 1
            ;;
        *)
            echo "ERROR: The script doesn't accept positional arguments"
            exit 1
            ;;
    esac
done
