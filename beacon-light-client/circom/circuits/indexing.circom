pragma circom 2.0.3;
include "../../../node_modules/circomlib/circuits/comparators.circom";

template Indexing(choices) {
  signal input array[choices];

  signal input index;

  signal output out;

  component lessThan = LessThan(32);
  lessThan.in[0] <== index;
  lessThan.in[1] <== choices;
  lessThan.out === 1;

  component eqs[choices];
  var sum = 0;

  for (var i = 0; i < choices; i ++) {
      eqs[i] = IsEqual();
      eqs[i].in[0] <== i;
      eqs[i].in[1] <== index;

      sum += eqs[i].out * array[i];
  }

  out <== sum;
}


component main = Indexing(512);
