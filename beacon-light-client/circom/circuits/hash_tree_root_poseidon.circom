pragma circom 2.1.5;

include "../../../node_modules/circomlib/circuits/poseidon.circom";

template HashTreeRootPoseidon(N) {
  signal input leaves[N];
  signal output out;

  component hashers[N - 1];

  for(var i = 0; i < N - 1; i++) {
    hashers[i] = Poseidon(2);
  }

  for(var i = 0; i < N / 2; i++) {
    hashers[i].inputs[0] <== leaves[i * 2];
    hashers[i].inputs[1] <== leaves[i * 2 + 1];
  }

  var k = 0;
  for(var i = N / 2; i < N - 1; i++) {
    hashers[i].inputs[0] <== hashers[k * 2].out;
    hashers[i].inputs[1] <== hashers[k * 2 + 1].out;

    k++;
  }

  out <== hashers[N - 2].out;
}
