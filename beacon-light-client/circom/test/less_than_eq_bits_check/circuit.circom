pragma circom 2.1.5;

include "../../circuits/utils/arrays.circom";

<<<<<<< HEAD
<<<<<<< HEAD
component main = LessThanOrEqualBitsCheck(32);
=======
component main = Selector(8) // N must be equal to input["in"] length
>>>>>>> 7f6ce53 (feat(circom) Add tests for Selector circuit.)
=======
component main = LessThanOrEqualBitsCheck(32) 
>>>>>>> 5452718 (fix(circom): Fix wrong main call for selector and less_than_eq_bits_check.)
