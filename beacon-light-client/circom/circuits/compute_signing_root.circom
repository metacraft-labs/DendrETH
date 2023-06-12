pragma circom 2.1.5;

include "compute_domain.circom";

template ComputeSigningRoot() {
  // In the "consensus-specs" they pass ssz-object(of type SSZObject) to "compute_signing_root"
  // then they hash it. We use the hash of the header(SSZObject) directly
  signal input headerHash[256];
  signal input domain[256];

  signal output signing_root[256];

  signal hashTwo[256] <== HashTwo()([headerHash,domain]);

  signing_root <== hashTwo;
}
