pragma circom 2.0.3;

include "../../../../vendor/circom-pairing/circuits/curve.circom";

template BlsAdd() {
  signal input first[2][7];
  signal input second[2][7];

  signal output out[2][7];

  component ellipticCurveAdd = EllipticCurveAdd(55, 7, 0, 4, [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425]);
  ellipticCurveAdd.aIsInfinity <== 0;
  ellipticCurveAdd.bIsInfinity <== 0;
  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      ellipticCurveAdd.a[j][k] <== first[j][k];
      ellipticCurveAdd.b[j][k] <== second[j][k];
    }
  }

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      out[j][k] <== ellipticCurveAdd.out[j][k];
    }
  }
}

component main = BlsAdd();
