pragma circom 2.0.3;

include "hash_two.circom";
include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";

template HashTwo() {
  signal input in[2][256];

  signal output out[256];

  component sha256 = Sha256(512);

  for(var i = 0; i < 256; i++) {
    sha256.in[i] <== in[0][i];
  }

  for(var i = 256; i < 512; i++) {
    sha256.in[i] <== in[1][i - 256];
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== sha256.out[i];
  }
}
