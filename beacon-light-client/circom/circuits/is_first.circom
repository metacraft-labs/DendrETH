pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/comparators.circom";
// include "../../../node_modules/circomlib/circuits/gates.circom";

template IsFirst() {
  signal input firstHash[2];
  signal input secondHash[2];

  signal output out;

  signal isEqual1 <== IsEqual()([firstHash[0],secondHash[0]]);

  signal isEqual2 <== IsEqual()([firstHash[1],secondHash[1]]);

  signal and <== AND()(isEqual1,isEqual2);

  out <== and;
}
