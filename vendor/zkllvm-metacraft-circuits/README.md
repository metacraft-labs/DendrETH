# zkLLVM template project for circuits compilation

Template repository for a zk-enabled application project
based on the [zkLLVM toolchain](https://github.com/nilfoundation/zkllvm).
The purpose of this project is to separate the compilation toolchain
from the application circuits. The tooling is added as submodule and we
have a src folder in the projects main folder where the new circuits will
be added. In order to build and run tests, currently docker or podman can
be used. All steps (build + run tests) can be performed by executing the 
script - scripts/compile_and_run_tests.sh.