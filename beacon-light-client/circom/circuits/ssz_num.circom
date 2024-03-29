pragma circom 2.1.5;

include "bitify.circom";

template SSZNum(N) {
  signal input in;

  signal output out[256];

  signal num2bits[N] <== Num2Bits(N)(in);

  var idx = N - 1;
  for(var i = N - 8; i >= 0; i -= 8) {
    for(var j = 0; j < 8; j++) {
      out[idx] <== num2bits[i + j];
      idx--;
    }
  }

  for(var j = N; j < 256; j++) {
    out[j] <== 0;
  }
}
