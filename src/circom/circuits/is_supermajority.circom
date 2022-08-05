pragma circom 2.0.3;

include "curve.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

template IsSuperMajority() {
  var N = 512;
  signal input bitmask[N];

}
