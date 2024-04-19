pragma circom 2.1.5;

include "comparators.circom";

template IsSuperMajority(N) {
  signal input bitmask[N];

  var sum = 0;
  //count the number of 1s in the bitmask
  for(var i = 0; i < N; i++) {
    sum += bitmask[i];
  }
  // check if 1s are more then 2/3 of the bitmask
  signal greaterThan <== GreaterEqThan(252)([sum * 3, 2 * N]);

  greaterThan === 1;
}
