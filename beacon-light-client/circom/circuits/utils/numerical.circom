pragma circom 2.1.5;

include "../../../../node_modules/circomlib/circuits/comparators.circom";
include "../../../../node_modules/circomlib/circuits/bitify.circom";
include "arrays.circom";

template LessThanBitsCheck(n) {
  signal input in[2];
  signal output out;

  signal bitCheck1[n] <== Num2Bits(n)(in[0]);

  signal bitCheck2[n] <== Num2Bits(n)(in[1]);

  out <== LessThan(n)(in);
}

template LessThanOrEqualBitsCheck(n) {
  signal input in[2];
  signal output out;

  signal bitCheck1[n] <== Num2Bits(n)(in[0]);

  signal bitCheck2[n] <== Num2Bits(n)(in[1]);

  out <== LessEqThan(n)(in);
}

template DivisionVerification() {
  signal input dividend;
  signal input divisor;
  signal input quotient;
  signal input remainder;

  dividend === divisor * quotient + remainder;
}

template Pow(N){
    signal input base;
    signal input power;
    signal output out;

    assert(power < N);

    signal intermediary[N];
    for (var i=0; i < N; i++) {
        intermediary[i] <== i == 0 ? 1 : (intermediary[i-1] * base);
    }
    
    component selector = Selector(N);
    for (var i = 0; i < N; i++) {
        selector.in[i] <== intermediary[i];
    }
    selector.index <== power;

    out <== selector.out;
}

component main = Pow(256);
