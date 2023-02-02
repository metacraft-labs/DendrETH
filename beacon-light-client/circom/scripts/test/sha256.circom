pragma circom 2.0.3;

include "../../../../node_modules/circomlib/circuits/sha256/sha256.circom";

template ShaTest() {
  signal input bits[1024];

  signal output out[256];


  component sha = Sha256(1024);

  for(var i = 0; i < 1024; i++) {
    sha.in[i] <== bits[i];
  }

  for (var i = 0; i < 256; i++) {
    out[i] <== sha.out[i];
  }
}

component main = ShaTest();
