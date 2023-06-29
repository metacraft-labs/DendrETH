pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template VerifySyncCommitee(N) {
  signal input root[256];
  signal input branch[5][256];
  signal input leaf[256];

}
