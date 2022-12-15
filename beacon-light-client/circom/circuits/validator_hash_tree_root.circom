pragma circom 2.0.3;

include "hash_two.circom";
include "ssz_num.circom";

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

  component slashedBits = SSZNum(8);
  slashedBits.in <== slashed;

  for(var j = 0; j < 256; j++) {
    hashers[1].in[0][j] <== effectiveBalance[j];
    hashers[1].in[1][j] <== slashedBits.out[j];
  }

  component activationEligibilityEpochBits = SSZNum(64);
  activationEligibilityEpochBits.in <== activationEligibilityEpoch;

  component activationEpochBits = SSZNum(64);
  activationEpochBits.in <== activationEpoch;

  for(var j = 0; j < 256; j++) {
    hashers[2].in[0][j] <== activationEligibilityEpochBits.out[j];
    hashers[2].in[1][j] <== activationEpochBits.out[j];
  }

  component exitEpochBits = SSZNum(64);
  exitEpochBits.in <== exitEpoch;

  for(var j = 0; j < 256; j++) {
    hashers[3].in[0][j] <== exitEpochBits.out[j];
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
