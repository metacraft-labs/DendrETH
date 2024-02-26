#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

CURRENT_DIR=$(pwd)

cd $SCRIPT_DIR/../docker && docker build -t zkllvm:latest -f Dockerfile .

cd $CURRENT_DIR

docker run --rm -it --name zk_executable_tests \
       --volume ${SCRIPT_DIR}/../zkllvm-template/:/zkllvm-template \
       --volume ${SCRIPT_DIR}/../../consensus-spec-tests/:/consensus-spec-tests \
       --volume ${SCRIPT_DIR}/../src/tests/verify_attestation_data_test/finalizer-data:/finalizer-data \
       --volume ${SCRIPT_DIR}/../src/:/zkllvm-template/src \
       --user $(id -u ${USER}):$(id -g ${USER}) \
       -w /zkllvm-template \
       zkllvm:latest \
       "$@"
