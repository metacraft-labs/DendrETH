# Weigh justification and finalization documentation.

### *Implementation*
   The implementation code of the circuit is under "DendrETH/vendor/zkllvm-metacraft-circuits/src/circuits_impl/weigh_justification_and_finalization_impl.h". 
   This code is used in both the compilation as circuit and as executable.

### *Circuit build*
   In order to build as circuit, we need an entry point marked with the `[[circuit]]` directive. This is done through a wrapper 
   that uses the implementation code and resides in "DendrETH/vendor/zkllvm-metacraft-circuits/src/circuits/weigh_justification_and_finalization.cpp".

### *Executable build + tests*
   The implementation code of the circuit is compiled as executable and tested against input data extracted from an Ethereum node. The tests reside in "DendrETH/vendor/zkllvm-metacraft-circuits/src/tests/weigh_justification_and_finalization_test".
   For convenience, we have a script that performs all necessary steps to run the test -> "DendrETH/vendor/zkllvm-metacraft-circuits/scripts/compile_and_run_tests.sh", which by default runs all tests. We can pass as argument to this script "weigh_justification_and_finalization_test" which will only run the relevant tests. For example, run the script from the main project directory "DendrETH/vendor/zkllvm-metacraft-circuits" as follows:
   `./scripts/compile_and_run_tests.sh weigh_justification_and_finalization_test`
   It is required that docker is installed on the machine that will run the tests.