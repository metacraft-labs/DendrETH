pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/comparators.circom";

template IsSuperMajority(N) {
  signal input bitmask[N];

  signal output out;

  var sum = 0;
  component lessThan[N];

  for(var i = 0; i < N; i++) {
    lessThan[i] = LessEqThan(1);
    lessThan[i].in[0] <== bitmask[i];
    lessThan[i].in[1] <== 1;

    lessThan[i].out === 1;

    sum += bitmask[i];
  }

  component greaterThan = GreaterEqThan(12);
  greaterThan.in[0] <== sum * 3;
  greaterThan.in[1] <== 1024;

  out <== greaterThan.out;
}
