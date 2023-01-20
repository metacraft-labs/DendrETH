pragma circom 2.0.3;

include "../../../node_modules/circomlib/circuits/gates.circom";

template XNOR() {
  signal input a;
  signal input b;

  signal output out;

  component xor = XOR();
  xor.a <== a;
  xor.b <== b;

  component not = NOT();
  not.in <== xor.out;

  out <== not.out;
}
