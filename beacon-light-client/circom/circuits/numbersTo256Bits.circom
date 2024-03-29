pragma circom 2.1.5;

include "bitify.circom";

template NumbersTo256Bits() {
  signal input first;
  signal input second;

  signal output out[256];

  signal num2bits1[253] <== Num2Bits(253)(first);

  signal num2bits2[3] <== Num2Bits(3)(second);

  for(var i = 0; i < 253; i++) {
    out[i] <== num2bits1[252 - i];
  }

  for(var i = 253; i < 256; i++) {
    out[i] <== num2bits2[255 - i];
  }
}
