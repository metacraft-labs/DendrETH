pragma circom 2.1.5;

include "comparators.circom";
include "bitify.circom";

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
