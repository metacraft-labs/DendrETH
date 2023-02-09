pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsSuperMajority(N) {
  signal input bitmask[N];

  signal output out;

  var sum = 0;

  for(var i = 0; i < N; i++) {
    sum += bitmask[i];
  }

  component greaterThan = GreaterEqThan(12);
  greaterThan.in[0] <== sum * 3;
  greaterThan.in[1] <== 2 * N;

  out <== greaterThan.out;
}
