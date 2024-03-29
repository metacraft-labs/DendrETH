pragma circom 2.1.5;

include "sha256/sha256.circom";
include "hash_tree_root.circom";
include "hash_aggregated_key.circom";

template SyncCommiteeHashTreeRoot(N) {
  signal input points[N][384];
  signal input aggregatedKey[384];

  signal output out[256];

  component leaves[N];
  signal hashTreeRootInput[N][256];

  for(var i = 0; i < N; i++) {
    leaves[i] = Sha256(512);
    for(var j = 0; j < 384; j++) {
      leaves[i].in[j] <== points[i][j];
    }

    for(var j = 384; j < 512; j++) {
      leaves[i].in[j] <== 0;
    }

    for(var j = 0; j < 256; j++) {
      hashTreeRootInput[i][j] <== leaves[i].out[j];
    }
  }

  signal hashTreeRoot[256] <== HashTreeRoot(N)(hashTreeRootInput);
  signal hashKey[256] <== hashAggregatedKey()(aggregatedKey);

  signal hasher[256] <== HashTwo()([hashTreeRoot,hashKey]);

  out <== hasher;
}
