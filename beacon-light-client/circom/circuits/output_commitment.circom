pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/mimcsponge.circom";

template OutputCommitment() {
  signal input currentEpoch;
  signal input participantsCount;
  signal input hash[256];
  signal input aggregatedKey[2][7];

  // verification key
  signal input negalfa1xbeta2[6][2][6]; // e(-alfa1, beta2)
  signal input gamma2[2][2][6];
  signal input delta2[2][2][6];
  signal input IC[2][2][6];

  signal output out;


  log(currentEpoch);
  log(participantsCount);
  for(var i = 0; i < 256; i++) {
    log(hash[i]);
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 7; j++) {
      log(aggregatedKey[i][j]);
    }
  }

  for(var i = 0; i < 6; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < 6; k++) {
        log(negalfa1xbeta2[i][j][k]);
      }
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < 6; k++) {
        log(gamma2[i][j][k]);
        log(delta2[i][j][k]);
        log(IC[i][j][k]);
      }
    }
  }
  component hasher = MiMCSponge(416, 220, 1);
  hasher.k <== 123;

  hasher.ins[0] <== currentEpoch;
  hasher.ins[1] <== participantsCount;

  for(var i = 0; i < 256; i++) {
    hasher.ins[2 + i] <== hash[i];
  }

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      hasher.ins[258 + j * 7 + k] <== aggregatedKey[j][k];
    }
  }

  var vkCounter = 272;

  for (var i = 0;i < 6;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.ins[vkCounter] <== negalfa1xbeta2[i][j][idx];
        vkCounter++;
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.ins[vkCounter] <== gamma2[i][j][idx];
        vkCounter++;
        hasher.ins[vkCounter] <== delta2[i][j][idx];
        vkCounter++;
        hasher.ins[vkCounter] <== IC[i][j][idx];
        vkCounter++;
      }
    }
  }

  out <== hasher.outs[0];
}
