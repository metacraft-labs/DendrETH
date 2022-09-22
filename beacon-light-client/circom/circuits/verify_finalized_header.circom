pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template VerifySyncCommitee(N) {
  signal input root[256];
  signal input branch[5][256];
  signal input leaf[256];

}
