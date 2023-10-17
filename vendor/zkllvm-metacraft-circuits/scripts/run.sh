#!/usr/bin/env bash

echo $@

COPY_SOURCE_FILES=1

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
REPO_ROOT="$SCRIPT_DIR/.."

# this function is called when Ctrl-C is sent
function trap_ctrlc ()
{
    # perform cleanup here
    echo "Ctrl-C caught...performing clean up"
 
    echo "Doing cleanup"
    if [ $COPY_SOURCE_FILES == 0 ]
    then
        mv $REPO_ROOT/src~ $REPO_ROOT/zkllvm-template/src
        cd $SCRIPT_DIR
        exit 1
    fi
}
 
# initialise trap to call trap_ctrlc function
# when signal 2 (SIGINT) is received
trap "trap_ctrlc" 2

echo "SCRIPT_DIR = " $SCRIPT_DIR
echo "REPO_ROOT = " $REPO_ROOT
ls -laht $REPO_ROOT/zkllvm-template/src
mv $REPO_ROOT/zkllvm-template/src $REPO_ROOT/src~
cp -r $REPO_ROOT/src $REPO_ROOT/zkllvm-template
cd $REPO_ROOT/zkllvm-template
COPY_SOURCE_FILES=0
scripts/run.sh $@
cd $SCRIPT_DIR/..
rm -r $REPO_ROOT/zkllvm-template/src
mv $REPO_ROOT/src~ $REPO_ROOT/zkllvm-template/src
