pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";

template HashTwo() {
  signal input in[2][256];

  signal output out[256];

  signal concateneted[512];

  for(var i = 0; i < 256; i++) {
    concateneted[i] <== in[0][i];
  }

  for(var i = 256; i < 512; i++) {
    concateneted[i] <== in[1][i - 256];
  }

  signal sha256[256] <== Sha256(512)(concateneted);

  out <== sha256;
}
