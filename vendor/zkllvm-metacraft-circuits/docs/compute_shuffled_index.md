# Compute shuffled index documentation.

### *Implementation*
   The implementation code of the circuit is under "DendrETH/vendor/zkllvm-metacraft-circuits/src/circuits_impl/compute_shuffled_index_impl.h". 
   This code is used in both the compilation as circuit and as executable.

### *Circuit build*
   In order to build as circuit, we need an entry point marked with the `[[circuit]]` directive. This is done through a wrapper 
   that uses the implementation code and resides in "DendrETH/vendor/zkllvm-metacraft-circuits/src/circuits/compute_shuffled_index.cpp". Since currently the Crypto3
   library does not implement computation of sha256 on a byte buffer, we use a header only library for the sha256 computations, which has 
   negative performance consequences.

### *Executable build + tests*
   The implementation code of the circuit is compiled as executable and tested against the input data from 
   https://github.com/ethereum/consensus-spec-tests.git. The tests reside in "DendrETH/vendor/zkllvm-metacraft-circuits/src/tests/compute_shuffled_index_test/".
   For convenience, we have a script that performs all necessary steps to run the test -> "DendrETH/vendor/zkllvm-metacraft-circuits/scripts/compile_and_run_tests.sh", which by default runs all tests. We can pass as argument to this script "compute_shuffled_index_test" which will only run the relevant tests. For example, run the script from the main project directory "DendrETH/vendor/zkllvm-metacraft-circuits" as follows:
   `./scripts/compile_and_run_tests.sh compute_shuffled_index_test`
   It is required that docker is installed on the machine that will run the tests.