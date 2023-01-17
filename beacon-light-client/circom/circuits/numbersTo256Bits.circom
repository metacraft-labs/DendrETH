pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/bitify.circom";

template NumbersTo256Bits() {
  signal input first;
  signal input second;

  signal output out[256];

  component num2bits1 = Num2Bits(253);
  num2bits1.in <== first;

  component num2bits2 = Num2Bits(3);
  num2bits2.in <== second;

  for(var i = 0; i < 253; i++) {
    out[i] <== num2bits1.out[252 - i];
  }

  for(var i = 253; i < 256; i++) {
    out[i] <== num2bits2.out[255 - i];
  }
}
