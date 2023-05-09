pragma circom 2.1.5;

include "hash_two.circom";
include "is_valid_merkle_branch_out.circom";
include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsValidMerkleBranch(N) {
  signal input branch[N][256];
  signal input leaf[256];
  signal input root[256];
  signal input index;

  signal isValidBalanceBranchOut <== IsValidMerkleBranchOut(N)(branch, leaf, root, index);

  isValidBalanceBranchOut === 1;
}
