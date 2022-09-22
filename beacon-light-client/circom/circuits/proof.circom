pragma circom 2.0.3;

include "hash_tree_root.circom";
include "compress.circom";
include "aggregate_bitmask.circom";
include "../../circuits/bls_signature.circom";

template Proof() {
  var K = 7;
  var N = 512;
  signal input points[N][2][K];
  signal input aggregatedKey[384];

  signal input bitmask[N];
  signal input signature[2][2][K];
  signal input hash[2][2][K];

  signal output hashTreeRoot[256];

  component hasher = HashTreeRoot();
  component compress[N];

  for(var i = 0; i < N; i++) {
    compress[i] = Compress();

    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        compress[i].point[j][k] <== points[i][j][k];
      }
    }

    for(var j = 0; j < 384; j++) {
      hasher.points[i][j] <== compress[i].bits[j];
    }
  }

  for(var i = 0; i < 384; i++) {
    hasher.aggregatedKey[i] <== aggregatedKey[i];
  }

  component aggregateKeys = AggregateKeysBitmask();

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        aggregateKeys.points[i][j][k] <== points[i][j][k];
      }
    }
  }

  for(var i = 0; i < N; i++) {
    aggregateKeys.bitmask[i] <== bitmask[i];
  }


  component verify = CoreVerifyPubkeyG1(55, K);

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < K; k++) {
      verify.pubkey[j][k] <== aggregateKeys.out[j][k];
      verify.signature[j][j][k] <== signature[j][j][k];
      verify.hash[j][j][k] <== hash[j][j][k];
    }
  }

  for(var i = 0; i < 256; i++) {
    hashTreeRoot[i] <== hasher.out[i];
  }
}
