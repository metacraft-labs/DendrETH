pragma circom 2.1.5;
include "gates.circom";
include "../../../../vendor/circom-pairing/circuits/bn254/groth16.circom";

template Test() {
  var pubInpCount = 1;
  // BN254 facts
  var k = 6;

  // public input
  signal input in;

  //
  signal input isFirst;

  // verification key
  signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input gamma2[2][2][k];
  signal input delta2[2][2][k];
  signal input IC[pubInpCount+1][2][k];

  // proof
  signal input negpa[2][k];
  signal input pb[2][2][k];
  signal input pc[2][k];
  signal input pubInput[pubInpCount];

  // check
  in === 1;

  // check recursive snark
  component groth16Verifier = verifyProof(pubInpCount);
  for (var i = 0;i < 6;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
          }
      }
  }
  for (var i = 0;i < 2;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.gamma2[i][j][idx] <== gamma2[i][j][idx];
              groth16Verifier.delta2[i][j][idx] <== delta2[i][j][idx];
              groth16Verifier.pb[i][j][idx] <== pb[i][j][idx];
          }
      }
  }
  for (var i = 0;i < pubInpCount + 1;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.IC[i][j][idx] <== IC[i][j][idx];
          }
      }
  }
  for (var i = 0;i < 2;i++) {
      for (var idx = 0;idx < k;idx++) {
          groth16Verifier.negpa[i][idx] <== negpa[i][idx];
          groth16Verifier.pc[i][idx] <== pc[i][idx];
      }
  }

  groth16Verifier.pubInput[0] <== in;

  component firstORcorrect = OR();
  firstORcorrect.a <== isFirst;
  firstORcorrect.b <== groth16Verifier.out;

  firstORcorrect.out === 1;
}

component main { public [ in ] } = Test();
