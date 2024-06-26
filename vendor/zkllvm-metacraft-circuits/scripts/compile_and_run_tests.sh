#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

CURRENT_DIR=$(pwd)

echo "CURRENT_DIR = " $CURRENT_DIR

cd $SCRIPT_DIR/../docker && docker build -t zcli:latest -f Dockerfile_zcli . \
                         && docker run -v $SCRIPT_DIR/../../:/DendrETH --user $(id -u ${USER}):$(id -g ${USER}) zcli:latest


FINALIZER_DATA_TEST_DIR=$SCRIPT_DIR/../src/tests/verify_attestation_data_test/finalizer-data

if [ ! -d $FINALIZER_DATA_TEST_DIR ]
then
    git clone git@github.com:metacraft-labs/finalizer-data.git $FINALIZER_DATA_TEST_DIR
fi

CONSENSUS_SPEC_TEST_DATA_DIR=$SCRIPT_DIR/../../consensus-spec-tests

if [ ! -d $CONSENSUS_SPEC_TEST_DATA_DIR ]
then
    git clone git@github.com:ethereum/consensus-spec-tests.git $CONSENSUS_SPEC_TEST_DATA_DIR
fi

cd $CURRENT_DIR

COMMAND='cmake -G "Unix Makefiles" -B ${ZKLLVM_BUILD:-build} -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=g++ . && make -C ${ZKLLVM_BUILD:-build} template'

${SCRIPT_DIR}/docker_run.sh "$COMMAND"

if [ $# == 0 ]
then
    bash ${SCRIPT_DIR}/docker_run.sh "make -C ${ZKLLVM_BUILD:-build} test "
else
    bash ${SCRIPT_DIR}/docker_run.sh "ctest --test-dir ${ZKLLVM_BUILD:-build} -R ${@}"
fi
