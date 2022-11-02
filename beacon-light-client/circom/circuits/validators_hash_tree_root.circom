pragma circom 2.0.3;

include "validator_hash_tree_root.circom";
include "hash_tree_root.circom";

template ValidatorsHashTreeRoot(N) {
  signal input pubkeys[N][384];
  signal input withdrawCredentials[N][256];

  signal input effectiveBalance[N][256];
  signal input slashed[N][256];

  signal input activationEligibilityEpoch[N][256];
  signal input activationEpoch[N][256];

  signal input exitEpoch[N][256];
  signal input withdrawableEpoch[N][256];
  signal output out[256];

  component pubkeyHashes[N];

  for(var i = 0; i < N; i++) {
    pubkeyHashes[i] = Sha256(512);

    for(var j = 0; j < 384; j++) {
      pubkeyHashes[i].in[j] <== pubkeys[i][j];
    }

    for(var j = 384; j < 512; j++) {
      pubkeyHashes[i].in[j] <== 0;
    }
  }

  component validatorHashers[N];

  for(var i = 0; i < N; i++) {
    validatorHashers[i] = ValidatorHashTreeRoot();
    for(var j = 0; j < 256; j++) {
      validatorHashers[i].pubkey[j] <== pubkeyHashes[i].out[j];
      validatorHashers[i].withdrawCredentials[j] <== withdrawCredentials[i][j];
      validatorHashers[i].effectiveBalance[j] <== effectiveBalance[i][j];
      validatorHashers[i].slashed[j] <== slashed[i][j];
      validatorHashers[i].activationEligibilityEpoch[j] <== activationEligibilityEpoch[i][j];
      validatorHashers[i].activationEpoch[j] <== activationEpoch[i][j];
      validatorHashers[i].exitEpoch[j] <== exitEpoch[i][j];
      validatorHashers[i].withdrawableEpoch[j] <== withdrawableEpoch[i][j];
    }
  }

  component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== validatorHashers[i].out[j];
    }
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== hashTreeRoot.out[i];
  }
}
