#!/usr/bin/env bash

echo "using nilfoundation/zkllvm-template:${ZKLLVM_VERSION:=0.0.86}"

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR = " $SCRIPT_DIR

CURRENT_DIR=$(pwd)

echo "CURRENT_DIR = " $CURRENT_DIR

cd $SCRIPT_DIR/../docker && docker build -f Dockerfile_zcli . && docker run -v $SCRIPT_DIR/../src:/DendrETH zcli:latest

cd $CURRENT_DIR

bash ${SCRIPT_DIR}/run.sh --docker compile

docker run --rm -it --name zk_executable_tests \
       --volume ${SCRIPT_DIR}/../zkllvm-template/build:/build \
       --user $(id -u ${USER}):$(id -g ${USER}) \
       -w /build \
       ghcr.io/nilfoundation/zkllvm-template:${ZKLLVM_VERSION} \
       make test
