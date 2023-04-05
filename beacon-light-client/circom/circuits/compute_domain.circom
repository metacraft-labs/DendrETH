pragma circom 2.0.3;

include "hash_two.circom";

template ComputeDomain() {
  signal input fork_version[32];
  signal output domain[256];

  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  component hashTwo = HashTwo();

  for(var i = 0; i < 32; i++) {
    hashTwo.in[0][i] <== fork_version[i];
  }

  for(var i = 32; i < 256; i++) {
    hashTwo.in[0][i] <== 0;
  }

  for(var i = 0; i < 256; i++) {
    hashTwo.in[1][i] <== GENESIS_VALIDATORS_ROOT[i];
  }

  for(var i = 0; i < 32; i++) {
    domain[i] <== DOMAIN_SYNC_COMMITTEE[i];
  }

  for(var i = 32; i < 256; i++) {
    domain[i] <== hashTwo.out[i - 32];
  }
}
