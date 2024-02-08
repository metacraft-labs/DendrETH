#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "SCRIPT_DIR = " $SCRIPT_DIR

CURRENT_DIR=$(pwd)

echo "CURRENT_DIR = " $CURRENT_DIR

cd $SCRIPT_DIR/../docker && docker build -t zcli:latest -f Dockerfile_zcli . && docker run -v $SCRIPT_DIR/../../:/DendrETH zcli:latest

if [ ! -d $SCRIPT_DIR/../src/tests/verify_attestation_data_test/finalizer-data ]
then 
    git clone git@github.com:metacraft-labs/finalizer-data.git $SCRIPT_DIR/../src/tests/verify_attestation_data_test/finalizer-data
fi

cd $CURRENT_DIR

bash ${SCRIPT_DIR}/run.sh --docker compile

bash ${SCRIPT_DIR}/docker_run.sh make test
