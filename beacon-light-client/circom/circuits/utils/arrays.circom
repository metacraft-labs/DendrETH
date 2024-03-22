pragma circom 2.1.5;

include "comparators.circom";
include "bitify.circom";

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
