pragma circom 2.0.3;

include "hash_two.circom";
include "merkle_root.circom";

template HashTreeRootBeaconHeader() {
  signal input slot[256];
  signal input proposer_index[256];
  signal input parent_root[256];
  signal input state_root[256];
  signal input body_root[256];

  signal output blockHash[256];

  component merkleRoot = MerkleRoot(8);

  for(var i = 0; i < 256; i++) {
    merkleRoot.leaves[0][i] <== slot[i];
  }

  for(var i = 0; i < 256; i++) {
    merkleRoot.leaves[1][i] <== proposer_index[i];
  }

  for(var i = 0; i < 256; i++) {
    merkleRoot.leaves[2][i] <== parent_root[i];
  }

  for(var i = 0; i < 256; i++) {
    merkleRoot.leaves[3][i] <== state_root[i];
  }

  for(var i = 0; i < 256; i++) {
    merkleRoot.leaves[4][i] <== body_root[i];
  }

  for(var i = 5; i < 8; i++) {
    for(var j = 0; j < 256; j++) {
      merkleRoot.leaves[i][j] <== 0;
    }
  }

  for(var i = 0; i < 256; i++) {
    blockHash[i] <== merkleRoot.root[i];
  }
}
