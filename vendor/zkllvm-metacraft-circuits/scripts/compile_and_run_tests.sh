#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR = " $SCRIPT_DIR

CURRENT_DIR=$(pwd)

echo "CURRENT_DIR = " $CURRENT_DIR

cd $SCRIPT_DIR/../docker && docker build -t zcli:latest -f Dockerfile_zcli . && docker run -v $SCRIPT_DIR/../../:/DendrETH zcli:latest

cd $CURRENT_DIR

bash ${SCRIPT_DIR}/run.sh --docker compile

bash ${SCRIPT_DIR}/docker_run.sh make test
