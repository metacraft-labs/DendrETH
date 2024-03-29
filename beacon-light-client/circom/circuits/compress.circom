pragma circom 2.1.5;

include "../../../vendor/circom-pairing/circuits/bigint.circom";

template Compress() {
  signal input point[2][7];

  signal output bits[384];

  // CURVE.P / 2
  var prime[7] = [35888059530597717, 36027359614205881, 18556878317960535, 21977360498475850, 26290126778424359, 29735955799434292, 914940731273212];

  signal lessThan <== BigLessThan(55, 7)(prime, point[1]);

  component num2Bits[7];

  for(var i = 0; i < 7; i++) {
    num2Bits[i] = Num2Bits(55);
    num2Bits[i].in <== point[0][i];

    for (var j = 0;(j < 55 && i < 6) || j < 51; j++) {
      bits[383 - i * 55 - j] <== num2Bits[i].out[j];
    }
  }

  bits[0] <== 1;
  bits[1] <== 0;
  bits[2] <== lessThan;
}
