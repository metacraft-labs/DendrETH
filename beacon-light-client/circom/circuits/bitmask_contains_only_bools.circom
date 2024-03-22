pragma circom 2.1.5;

include "comparators.circom";

template BitmaskContainsOnlyBools(N) {
  signal input bitmask[N];

  component lessThan[N];

  for(var i = 0; i < N; i++) {
    lessThan[i] = LessEqThan(1);
    lessThan[i].in[0] <== bitmask[i];
    lessThan[i].in[1] <== 1;

    lessThan[i].out === 1;
  }
}
