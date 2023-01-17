pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/bitify.circom";

template SSZNum(N) {
  signal input in;

  signal output out[256];

  component num2bits = Num2Bits(N);
  num2bits.in <== in;

  var idx = N - 1;
  for(var i = N - 8; i >= 0; i -= 8) {
    for(var j = 0; j < 8; j++) {
      out[idx] <== num2bits.out[i + j];
      idx--;
    }
  }

  for(var j = N; j < 256; j++) {
    out[j] <== 0;
  }
}
