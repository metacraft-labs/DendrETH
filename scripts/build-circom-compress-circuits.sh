#!/usr/bin/env bash

# ****CREATE BUILD FOLDER****
mkdir -p "${GIT_ROOT}/beacon-light-client/circom/build/compress"

# ****DOWNLOAD PTAU FILE****
curl https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau --output beacon-light-client/circom/build/compress/pot28_final.ptau

# ****RUN BUILDING SCRIPT****
"${GIT_ROOT}/beacon-light-client/circom/scripts/compress/compress.sh"
