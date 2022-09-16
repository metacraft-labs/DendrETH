pragma circom 2.0.3;

include "hash_two.circom";
include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsValidMerkleBranch(N) {
  signal input branch[N][256];
  signal input leaf[256];
  signal input root[256];
  signal input index;

  signal output out;

  component hashers[N];
  component isZero[N];

  for(var i = 0; i < N; i++) {
    hashers[i] = HashTwo();
    isZero[i] = IsZero();

    isZero[i].in <== (index / (2**i)) % 2;

    var current[256];

    for(var j = 0; j < 256; j++) {
      current[j] = i == 0 ? leaf[j] : hashers[i].out[j];
    }

    for(var j = 0; j < 256; j++) {
      hashers[i].in[0][j] <== (branch[i][j] - current[j]) * isZero[i].out + current[j];
      hashers[i].in[1][j] <== (current[i] - branch[i][j]) * isZero[i].out + branch[i][j];
    }
  }

  var counter = 0;
  component isEqual[N+1];
  for(var i = 0; i < 256; i++) {
    isEqual[i] = IsEqual();
    isEqual[i].in[0] <== root[i];
    isEqual[i].in[1] <== hashers[255].out[i];
    counter += isEqual[i].out;
  }

  isEqual[256] = IsEqual();

  isEqual[256].in[0] <== 256;
  isEqual[256].in[1] <== counter;

  out <== isEqual[256].out;
}
