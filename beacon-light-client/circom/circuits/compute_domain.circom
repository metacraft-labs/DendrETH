pragma circom 2.1.5;

include "hash_two.circom";

template ComputeDomain() {
  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal output domain[256];

  signal concated_fork_version[256];

  for(var i = 0; i < 32; i++) {
    concated_fork_version[i] <== fork_version[i];
  }
  for(var i = 32; i < 256; i++) {
    concated_fork_version[i] <== 0;
  }

  signal hashTwo[256] <== HashTwo()([concated_fork_version,GENESIS_VALIDATORS_ROOT]);

  for(var i = 0; i < 32; i++) {
    domain[i] <== DOMAIN_SYNC_COMMITTEE[i];
  }

  for(var i = 32; i < 256; i++) {
    domain[i] <== hashTwo[i - 32];
  }
}
