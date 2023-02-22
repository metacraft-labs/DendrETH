pragma circom 2.0.3;

include "compute_domain.circom";

template ComputeSigningRoot() {
  // In the "consensus-specs" they pass ssz-object(of type SSZObject) to "compute_signing_root"
  // then they hash it. We use the hash of the header(SSZObject) directly
  signal input headerHash[256];
  signal input domain[256];

  signal output signing_root[256];

  component hashTwo = HashTwo();

  for(var i = 0; i < 256; i++) {
    hashTwo.in[0][i] <== headerHash[i];
  }

  for(var i = 0; i < 256; i++) {
    hashTwo.in[1][i] <== domain[i];
  }


  for(var i = 0; i < 256; i++) {
    signing_root[i] <== hashTwo.out[i];
  }
}
