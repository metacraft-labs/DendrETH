pragma circom 2.1.5;

include "hash_two.circom";
include "hash_tree_root.circom";

template HashTreeRootBeaconHeader() {
  signal input slot[256];
  signal input proposer_index[256];
  signal input parent_root[256];
  signal input state_root[256];
  signal input body_root[256];

  signal output out[256];

  signal zerosArr[3][256];

  for(var i = 0; i < 3; i++) {
    for(var j = 0; j < 256; j++) {
      zerosArr[i][j] <== 0;
    }
  }

  signal hashTreeRoot[256] <== HashTreeRoot(8)([slot, proposer_index,
  parent_root, state_root, body_root, zerosArr[0], zerosArr[1], zerosArr[2]]);

  out <== hashTreeRoot;
}
