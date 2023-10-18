# zkLLVM template project for circuits compilation

Template repository for a zk-enabled application project
based on the [zkLLVM toolchain](https://github.com/nilfoundation/zkllvm).
The purpose of this project is to separate the compilation toolchain
from the application circuits. The tooling is added as submodule and we
have a src folder in the projects main folder where the new circuits will
be added. To compile them, use the run.sh scrypt in folder
zkllvm-metacraft-circuits/scripts passing the arguments you would pass to
zkllvm-metacraft-circuits/zkllvm-template/scripts/run.sh.