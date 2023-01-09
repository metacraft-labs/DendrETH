pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/pedersen.circom";

template HashTreeRootPedersen(N) {
  signal input leaves[N];

  signal output out;

  component hashers[N - 1];

  for(var i = 0; i < N - 1; i++) {
    hashers[i] = Pedersen(2);
  }

  for(var i = 0; i < N / 2; i++) {
    hashers[i].in[0] <== leaves[i * 2];
    hashers[i].in[1] <== leaves[i * 2 + 1];
  }

  var k = 0;

  for(var i = N / 2; i < N - 1; i++) {
    hashers[i].in[0] <== hashers[k * 2].out[0];
    hashers[i].in[1] <== hashers[k * 2 + 1].out[0];

    k++;
  }

  out <== hashers[N - 2].out[0];
}
