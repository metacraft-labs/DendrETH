pragma circom 2.0.3;
include "hash_two.circom";

template ValidatorHashTreeRoot() {
  signal input pubkey[256];
  signal input withdrawCredentials[256];

  signal input effectiveBalance[256];
  signal input slashed[256];

  signal input activationEligibilityEpoch[256];
  signal input activationEpoch[256];

  signal input exitEpoch[256];
  signal input withdrawableEpoch[256];

  signal output out[256];

  component hashers[7];

  for(var i = 0; i < 7; i++) {
    hashers[i] = HashTwo();
  }

  for(var j = 0; j < 256; j++) {
    hashers[0].in[0][j] <== pubkey[j];
    hashers[0].in[1][j] <== withdrawCredentials[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[1].in[0][j] <== effectiveBalance[j];
    hashers[1].in[1][j] <== slashed[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[2].in[0][j] <== activationEligibilityEpoch[j];
    hashers[2].in[1][j] <== activationEpoch[j];
  }

  for(var j = 0; j < 256; j++) {
    hashers[3].in[0][j] <== exitEpoch[j];
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
