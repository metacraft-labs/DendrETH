pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/pedersen.circom";

template OutputCommitment() {
  signal input minExitEpoch;
  signal input maxActivationEpoch;
  signal input hash;
  signal input aggregatedKey[2][7];

  // verification key
  signal input negalfa1xbeta2[6][2][6]; // e(-alfa1, beta2)
  signal input gamma2[2][2][6];
  signal input delta2[2][2][6];
  signal input IC[2][2][6];

  signal output out;

  component hasher = Pedersen(161);

  hasher.in[0] <== minExitEpoch;
  hasher.in[1] <== maxActivationEpoch;
  hasher.in[2] <== hash;

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      hasher.in[3 + j * 7 + k] <== aggregatedKey[j][k];
    }
  }

  for (var i = 0;i < 6;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.in[17 + i * 12 + j * 6 + idx] <== negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.in[89 + i * 12 + j * 6 + idx] <== gamma2[i][j][idx];
        hasher.in[113 + i * 12 + j * 6 + idx] <== delta2[i][j][idx];
        hasher.in[137 + i * 12 + j * 6 + idx] <== IC[i][j][idx];
      }
    }
  }

  out <== hasher.out[0];
}
