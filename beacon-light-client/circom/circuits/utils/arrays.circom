pragma circom 2.1.5;

include "../../../../node_modules/circomlib/circuits/comparators.circom";
include "../../../../node_modules/circomlib/circuits/bitify.circom";

template RangeCheck(n) {
  signal input in[3];
  signal output out;

  signal first <== LessThanBitsCheck(64)([in[0], in[1]]);
  signal second <== LessThanBitsCheck(64)([in[1], in[2]]);

  out <== first * second;
}

template Selector(N) {
    signal input in[N];
    signal input index;
    signal output out;

    signal sums[N + 1];
    sums[0] <== 0;

    component eqs[N];

    // For each item, check whether its index equals the input index.
    for (var i = 0; i < N; i ++) {
        eqs[i] = IsEqual();
        eqs[i].in[0] <== i;
        eqs[i].in[1] <== index;

        // eqs[i].out is 1 if the index matches. As such, at most one input to
        sums[i + 1] <== sums[i] + eqs[i].out * in[i];
    }

    // Returns 0 + 0 + ... + item
    out <== sums[N];
}

template IsEqualArrays(N) {
  signal input in[2][N];
  signal output out;

  signal isEqual[N];
  var counter = 0;

  for(var i = 0; i < N; i++) {
    isEqual[i] <== IsEqual()([in[0][i], in[1][i]]);

    counter += isEqual[i];
  }

  out <== IsEqual()([N, counter]);
}
