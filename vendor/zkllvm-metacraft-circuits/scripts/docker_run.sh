#!/usr/bin/env bash

echo "using ghcr.io/nilfoundation/toolchain:${ZKLLVM_VERSION:=0.1.8}"

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

docker run --rm -it --name zk_executable_tests \
       --volume ${SCRIPT_DIR}/../zkllvm-template/build:/build \
       --volume ${SCRIPT_DIR}/../../consensus-spec-tests/:/consensus-spec-tests \
       --user $(id -u ${USER}):$(id -g ${USER}) \
       -w /build \
       ghcr.io/nilfoundation/toolchain:${ZKLLVM_VERSION} \
       $@