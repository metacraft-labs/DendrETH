pragma circom 2.0.0;

template Pow() {
  var N = 12;
    signal input a;
    signal output out;

    signal sums[N];
    sums[0] <== a;

    for(var i = 1; i < N; i++) {
      sums[i] <== sums[i - 1] * a;
    }

    out <== sums[N - 1];
 }

 component main = Pow();
