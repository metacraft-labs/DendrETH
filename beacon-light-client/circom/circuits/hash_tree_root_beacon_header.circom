pragma circom 2.0.3;

include "hash_two.circom";
include "hash_tree_root.circom";

template HashTreeRootBeaconHeader() {
  signal input slot[256];
  signal input proposer_index[256];
  signal input parent_root[256];
  signal input state_root[256];
  signal input body_root[256];

  signal output out[256];

  component hashTreeRoot = HashTreeRoot(8);

  for(var i = 0; i < 256; i++) {
    hashTreeRoot.leaves[0][i] <== slot[i];
  }

  for(var i = 0; i < 256; i++) {
    hashTreeRoot.leaves[1][i] <== proposer_index[i];
  }

  for(var i = 0; i < 256; i++) {
    hashTreeRoot.leaves[2][i] <== parent_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hashTreeRoot.leaves[3][i] <== state_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hashTreeRoot.leaves[4][i] <== body_root[i];
  }

  for(var i = 5; i < 8; i++) {
    for(var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== 0;
    }
  }

  for(var i = 0; i < 256; i++) {
    out[i] <== hashTreeRoot.out[i];
  }
}
