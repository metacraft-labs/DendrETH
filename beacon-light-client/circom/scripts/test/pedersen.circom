pragma circom 2.0.3;

include "../../../../node_modules/circomlib/circuits/pedersen.circom";

template Pedersen1024() {
  signal input bits[1024];

  signal output out[2];

  component pedersen = Pedersen(1024);

  for (var i = 0; i < 1024; i++) {
    pedersen.in[i] <== bits[i];
  }

  out[0] <== pedersen.out[0];
  out[1] <== pedersen.out[1];
}

component main = Pedersen1024();
