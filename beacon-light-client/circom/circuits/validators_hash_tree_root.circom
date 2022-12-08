pragma circom 2.0.3;

include "validator_hash_tree_root.circom";
include "hash_tree_root.circom";

template ValidatorsHashTreeRoot(N) {
  signal input pubkeys[N][384];
  signal input withdrawCredentials[N][256];

  signal input effectiveBalance[N][256];
  signal input slashed[N];

  signal input activationEligibilityEpoch[N];
  signal input activationEpoch[N];

  signal input exitEpoch[N];
  signal input withdrawableEpoch[N][256];

  signal input zero[N];

  signal output out[256];

  component validatorHashers[N];

  for(var i = 0; i < N; i++) {
    validatorHashers[i] = ValidatorHashTreeRoot();
    for(var j = 0; j < 384; j++) {
      validatorHashers[i].pubkey[j] <== pubkeys[i][j];
    }

    for(var j = 0; j < 256; j++) {
      validatorHashers[i].withdrawCredentials[j] <== withdrawCredentials[i][j];
      validatorHashers[i].effectiveBalance[j] <== effectiveBalance[i][j];
      validatorHashers[i].withdrawableEpoch[j] <== withdrawableEpoch[i][j];
    }

    validatorHashers[i].slashed <== slashed[i];
    validatorHashers[i].activationEligibilityEpoch <== activationEligibilityEpoch[i];
    validatorHashers[i].activationEpoch <== activationEpoch[i];
    validatorHashers[i].exitEpoch <== exitEpoch[i];
  }

  component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== validatorHashers[i].out[j] * zero[i];
    }
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== hashTreeRoot.out[i];
  }
}
