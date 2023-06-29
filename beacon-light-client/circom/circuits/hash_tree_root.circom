pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";
include "hash_two.circom";

template HashTreeRoot(N) {
  signal input leaves[N][256];

  signal output out[256];

  component hashers[N - 1];

  for(var i = 0; i < N - 1; i++) {
    hashers[i] = HashTwo();
  }

  for(var i = 0; i < N / 2; i++) {
    hashers[i].in[0] <== leaves[i * 2];
    hashers[i].in[1] <== leaves[i * 2 + 1];
  }

  var k = 0;

  for(var i = N / 2; i < N - 1; i++) {
    hashers[i].in[0] <== hashers[k * 2].out;
    hashers[i].in[1] <== hashers[k * 2 + 1].out;

    k++;
  }

  out <== hashers[N - 2].out;
}
