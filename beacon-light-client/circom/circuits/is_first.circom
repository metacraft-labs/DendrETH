pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsFirst() {
  signal input firstHash[2];
  signal input secondHash[2];

  signal output out;

  component isEqual1 = IsEqual();
  isEqual1.in[0] <== firstHash[0];
  isEqual1.in[1] <== secondHash[0];

  component isEqual2 = IsEqual();
  isEqual2.in[0] <== firstHash[1];
  isEqual2.in[1] <== secondHash[1];

  component and = AND();

  and.a <== isEqual1.out;
  and.b <== isEqual2.out;

  out <== and.out;
}
