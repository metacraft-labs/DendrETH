pragma circom 2.0.0;

template QuadraticEquation() {
    signal input x;
    signal output e;

    e <== x * x - 4 * x + 7;
 }

 component main = QuadraticEquation();
