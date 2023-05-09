pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/comparators.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";

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

template RangeCheck(n) {
  signal input in[3];
  signal output out;

  signal first <== LessThanBitsCheck(64)([in[0], in[1]]);
  signal second <== LessThanBitsCheck(64)([in[1], in[2]]);

  out <== first * second;
}

template IsEqualArrays(N) {
  signal input in[2][N];
  signal output out;

  signal isEqual[N];
  var counter = 0;

  for(var i = 0; i < N; i++) {
    isEqual[i] <== IsEqual()([in[0][i], in[1][i]]);

    counter += isEqual[i];
  }

  out <== IsEqual()([N, counter]);
}
