pragma circom 2.0.3;

include "../../../../vendor/circom-pairing/circuits/bls_signature.circom";

template VerifySignature() {
  signal input publicKey[2][7];
  signal input signature[2][2][7];
  signal input  hash[2][2][7];

  component verify = CoreVerifyPubkeyG1(55, 7);

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      verify.pubkey[j][k] <== publicKey[j][k];
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < 7; k++) {
        verify.signature[i][j][k] <== signature[i][j][k];
        verify.hash[i][j][k] <== hash[i][j][k];
      }
    }
  }
}


component main = VerifySignature();
