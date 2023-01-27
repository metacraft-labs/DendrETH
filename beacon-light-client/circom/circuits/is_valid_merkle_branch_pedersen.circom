pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/pedersen.circom";
include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsValidMerkleBranchPedersen(N) {
  signal input branch[N];
  signal input leaf;
  signal input root;
  signal input index;

  signal output out;

  component hashers[N];
  component isZero[N];

  for(var i = 0; i < N; i++) {
    hashers[i] = Pedersen(2);
    isZero[i] = IsZero();

    isZero[i].in <-- (index \ (2**i)) % 2;

    var current;

    current = i == 0 ? leaf : hashers[i - 1].out[0];

    hashers[i].in[0] <== (current - branch[i]) * isZero[i].out + branch[i];
    hashers[i].in[1] <== (branch[i] - current) * isZero[i].out + current;
  }

  component isEqual = IsEqual();

  isEqual.in[0] <== root;
  isEqual.in[1] <== hashers[N-1].out[0];

  out <== isEqual.out;
}
