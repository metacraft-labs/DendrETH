pragma circom 2.0.3;
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "hash_two.circom";

template ValidatorHashTreeRoot() {
  signal input pubkey[384];
  signal input withdrawCredentials[256];

  signal input effectiveBalance[256];
  signal input slashed;

  signal input activationEligibilityEpoch;
  signal input activationEpoch;

  signal input exitEpoch;
  signal input withdrawableEpoch[256];

  signal output out[256];

  component hashers[7];

  component pubkeyHasher = Sha256(512);

  for(var i = 0; i < 384; i++) {
    pubkeyHasher.in[i] <== pubkey[i];
  }

  for(var i = 384; i < 512; i++) {
    pubkeyHasher.in[i] <== 0;
  }

  for(var i = 0; i < 7; i++) {
    hashers[i] = HashTwo();
  }

  for(var j = 0; j < 256; j++) {
    hashers[0].in[0][j] <== pubkeyHasher.out[j];
    hashers[0].in[1][j] <== withdrawCredentials[j];
  }

  hashers[1].in[1][0] <== slashed;
  for(var j = 0; j < 256; j++) {
    hashers[1].in[0][j] <== effectiveBalance[j];
    if(j > 0) {
      hashers[1].in[1][j] <== 0;
    }
  }

  component activationEligibilityEpochBits = Num2Bits(64);
  activationEligibilityEpochBits.in <== activationEligibilityEpoch;

  component activationEpochBits = Num2Bits(64);
  activationEpochBits.in <== activationEpoch;

  var idx = 63;
  for(var i = 56; i >= 0; i -= 8) {
    for(var j = 0; j < 8; j++) {
      hashers[2].in[0][idx] <== activationEligibilityEpochBits.out[i + j];
      hashers[2].in[1][idx] <== activationEpochBits.out[i + j];
      idx--;
    }
  }

  for(var j = 64; j < 256; j++) {
    hashers[2].in[0][j] <== 0;
    hashers[2].in[1][j] <== 0;
  }

  component exitEpochBits = Num2Bits(64);
  exitEpochBits.in <== exitEpoch;

  idx = 63;
  for(var i = 56; i >= 0; i-=8) {
    for(var j = 0; j < 8; j++) {
      hashers[3].in[0][idx] <== exitEpochBits.out[i + j];
      idx--;
    }
  }

  for(var j = 64; j < 256; j++) {
    hashers[3].in[0][j] <== 0;
  }

  for(var j = 0; j < 256; j++) {
    hashers[3].in[1][j] <== withdrawableEpoch[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[4].in[0][j] <== hashers[0].out[j];
    hashers[4].in[1][j] <== hashers[1].out[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[5].in[0][j] <== hashers[2].out[j];
    hashers[5].in[1][j] <== hashers[3].out[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[6].in[0][j] <== hashers[4].out[j];
    hashers[6].in[1][j] <== hashers[5].out[j];
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== hashers[6].out[i];
  }
}
