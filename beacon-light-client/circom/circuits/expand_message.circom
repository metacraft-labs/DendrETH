pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";

template ExpandMessage() {
  signal input in[256];
  signal output out[2048];

  component b_0Sha = Sha256(1144);

  for(var i = 0; i < 512; i++) {
    b_0Sha.in[i] <== 0;
  }

  var BIG_SIG_DST[352] = [0,1,0,0,0,0,1,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,1,1,0,1,0,1,1,1,1,1,0,1,0,1,0,0,1,1,0,1,0,0,1,0,0,1,0,1,0,0,0,1,1,1,0,1,0,1,1,1,1,1,0,1,0,0,0,0,1,0,0,1,0,0,1,1,0,0,0,1,0,1,0,0,1,1,0,0,1,1,0,0,0,1,0,0,1,1,0,0,1,0,0,0,1,1,0,0,1,1,0,0,1,1,1,0,0,0,0,0,1,1,0,0,0,1,0,1,0,0,0,1,1,1,0,0,1,1,0,0,1,0,0,1,0,1,1,1,1,1,0,1,0,1,1,0,0,0,0,1,0,0,1,1,0,1,0,1,0,0,0,1,0,0,0,0,1,1,1,0,1,0,0,1,0,1,0,0,1,1,0,1,0,0,1,0,0,0,0,1,0,0,0,0,0,1,0,0,1,0,1,1,0,1,0,0,1,1,0,0,1,0,0,0,1,1,0,1,0,1,0,0,1,1,0,1,1,0,0,1,0,1,1,1,1,1,0,1,0,1,0,0,1,1,0,1,0,1,0,0,1,1,0,1,0,1,0,1,1,1,0,1,0,1,0,1,0,1,0,1,0,1,1,1,1,1,0,1,0,1,0,0,1,0,0,1,0,0,1,1,1,1,0,1,0,1,1,1,1,1,0,1,0,1,0,0,0,0,0,1,0,0,1,1,1,1,0,1,0,1,0,0,0,0,0,1,0,1,1,1,1,1,0,0,1,0,1,0,1,1];

  for (var i = 512; i < 768; i++) {
    b_0Sha.in[i] <== in[i - 512];
  }
  for(var i = 768; i < 775; i++) {
    b_0Sha.in[i] <== 0;
  }

  b_0Sha.in[775] <== 1;
  for(var i = 776; i < 792; i++) {
    b_0Sha.in[i] <== 0;
  }

  for(var i = 792; i < 1144; i++) {
    b_0Sha.in[i] <== BIG_SIG_DST[i - 792];
  }

  var b_0[256] = b_0Sha.out;

  component prevSha256[8];

  prevSha256[0] = Sha256(616);

  for(var i = 0; i < 256; i++) {
    prevSha256[0].in[i] <== b_0[i];
  }

  for(var i =256; i < 263; i++){
    prevSha256[0].in[i] <== 0;
  }

  prevSha256[0].in[263] <== 1;

  for(var i = 264; i < 616; i++) {
    prevSha256[0].in[i] <== BIG_SIG_DST[i - 264];
  }

  for(var i = 0; i < 256; i++){
    out[i] <== prevSha256[0].out[i];
  }

  component numbits[7];

  for(var index = 1; index < 8; index++) {
    prevSha256[index] = Sha256(616);

    for(var i = 0; i < 256; i++) {
      // xor
      prevSha256[index].in[i] <== b_0[i] + prevSha256[index - 1].out[i] - 2 * b_0[i] * prevSha256[index - 1].out[i];
    }

    numbits[index - 1] = Num2Bits(8);

    numbits[index - 1].in <== index + 1;

    for(var i = 256; i < 264; i++) {
      prevSha256[index].in[i] <== numbits[index - 1].out[263 - i];
    }

    for(var i = 264; i < 616; i++) {
      prevSha256[index].in[i] <== BIG_SIG_DST[i - 264];
    }

    for(var i = 0; i < 256; i++){
      out[index * 256 + i] <== prevSha256[index].out[i];
    }
  }
}
