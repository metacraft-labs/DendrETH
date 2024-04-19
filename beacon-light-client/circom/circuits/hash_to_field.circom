pragma circom 2.1.5;

include "expand_message.circom";
include "../../../vendor/circom-pairing/circuits/bigint.circom";

template HashToField(K) {
  signal input in[256];
  signal output out[2][2][K];

  signal expand_message[2048] <== ExpandMessage()(in);

  component bigInts[2][2][10];

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      bigInts[i][j][9] = Bits2Num(55);
      for(var i1=0; i1 < 17; i1++) {
        bigInts[i][j][9].in[16 - i1] <== expand_message[i * 1024 + j * 512 + i1];
      }

      for(var i1 = 17; i1 < 55; i1++) {
        bigInts[i][j][9].in[i1] <== 0;
      }

      for(var k = 8; k >= 0; k--) {
        bigInts[i][j][k] = Bits2Num(55);
        for(var i1 = 0; i1 < 55; i1++) {
          bigInts[i][j][k].in[54 - i1] <== expand_message[i * 1024 + j * 512 + (8-k) * 55 + i1 + 17];
        }
      }
    }
  }

  component bigMod[2][2];

  var p[7] = [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425];

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      bigMod[i][j] = BigMod(55, 7);

      for(var k = 0; k < 10; k++) {
        bigMod[i][j].a[k] <== bigInts[i][j][k].out;
      }

      for(var k = 10; k < 14; k++){
        bigMod[i][j].a[k] <== 0;
      }

      bigMod[i][j].b <== p;

    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      out[i][j] <== bigMod[i][j].mod;
    }
  }
}
