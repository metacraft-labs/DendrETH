pragma circom 2.1.5;

include "../../../vendor/circom-pairing/circuits/curve.circom";

template AggregateKeysBitmask(N,K) {
  var J = 2;
  signal input points[N][J][K];
  signal input bitmask[N];

  // generator point used here but it should not matter https://github.com/yi-sun/circom-pairing/issues/12#issuecomment-1219292306
  var zero[2][7] = [[16589478066046651, 22658679592837110, 35004527604248919, 16789302793630161, 7530538873701625, 32234187716135413, 1684953952445941], [11860609209853921, 4091579406338073, 12578493816062195, 13088963032438982, 24961455755233629,8501508834176643, 612415636564648]];

  signal output out[J][K];

  component ellipticCurveAdd[N];

  var counter = 0;

  ellipticCurveAdd[0] = EllipticCurveAdd(55, 7, 0, 4, [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425]);
  ellipticCurveAdd[0].aIsInfinity <== 1;
  ellipticCurveAdd[0].bIsInfinity <== -1 * bitmask[0] + 1;
  counter += bitmask[0];
  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      ellipticCurveAdd[0].a[j][k] <== zero[j][k];
      ellipticCurveAdd[0].b[j][k] <== (-1 * bitmask[0] + 1) * (zero[j][k] - points[0][j][k]) + points[0][j][k];
    }
  }

  component lessThan[N];
  for(var i = 1; i < N; i++){
    ellipticCurveAdd[i] = EllipticCurveAdd(55, 7, 0, 4, [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425]);
    lessThan[i] = LessThan(56);
    lessThan[i].in[0] <== counter;
    lessThan[i].in[1] <== 1;
    var aIsInfinity = lessThan[i].out;
    var bIsInfinity = (-1 * (bitmask[i]) + 1);
    ellipticCurveAdd[i].aIsInfinity <== aIsInfinity;
    ellipticCurveAdd[i].bIsInfinity <== bIsInfinity;
    counter += bitmask[i];
    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        ellipticCurveAdd[i].a[j][k] <== aIsInfinity * (zero[j][k] - ellipticCurveAdd[i - 1].out[j][k]) + ellipticCurveAdd[i - 1].out[j][k];
        ellipticCurveAdd[i].b[j][k] <== bIsInfinity * (zero[j][k] - points[i][j][k]) + points[i][j][k];
      }
    }
  }

  for(var j = 0; j < J; j++){
    for(var k = 0; k < K; k++) {
      out[j][k] <== ellipticCurveAdd[N - 1].out[j][k];
    }
  }
}
