pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/sha256/sha256.circom";
include "hash_tree_root.circom";

template SyncCommiteeHashTreeRoot(N) {
  signal input points[N][384];
  signal input aggregatedKey[384];

  signal output out[256];

  component leaves[N];

  for(var i = 0; i < N; i++) {
    // SSZ pubkey
    leaves[i] = Sha256(512);
    for(var j = 0; j < 384; j++) {
      leaves[i].in[j] <== points[i][j];
    }

    for(var j = 384; j < 512; j++) {
      leaves[i].in[j] <== 0;
    }
  }

  component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== leaves[i].out[j];
    }
  }

  // SSZ pubkey
  component hash = Sha256(512);

  for(var i = 0; i < 384; i++) {
    hash.in[i] <== aggregatedKey[i];
  }

  for(var i = 384; i < 512; i++) {
    hash.in[i] <== 0;
  }

  component hasher = HashTwo();

  for(var i = 0; i < 256; i++) {
    hasher.in[0][i] <== hashTreeRoot.out[i];
    hasher.in[1][i] <== hash.out[i];
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== hasher.out[i];
  }
}
