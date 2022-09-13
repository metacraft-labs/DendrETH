pragma circom 2.0.3;

include "hash_tree_root.circom";
include "compress.circom";
include "aggregate_bitmask.circom";
include "../../../vendor/circom-pairing/circuits/bls_signature.circom";

template ProofMoreEfficient(N) {
  var K = 7;
  signal input points[N][2][K];
  signal input aggregatedKey[384];
  signal input bitmask[N];
  signal input signature[2][2][K];

  signal input hash[2][2][K];
  signal output hashTreeRoot[2];


  component isSuperMajority = IsSuperMajority(N);

  for(var i = 0; i < N; i++) {
    isSuperMajority.bitmask[i] <== bitmask[i];
  }

  isSuperMajority.out === 1;


  component hasher = HashTreeRoot(N);
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

  component aggregateKeys = AggregateKeysBitmask(N);


  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        aggregateKeys.points[i][j][k] <== points[i][j][k];
      }
    }
  }

  component bitmaskNum2Bits[3];

  bitmaskNum2Bits[0] = Num2Bits(6);
  bitmaskNum2Bits[0].in <== bitmask[0];

  for(var i = 0; i < 6; i++) {
    aggregateKeys.bitmask[i] <== bitmaskNum2Bits[0].out[i];
  }

  bitmaskNum2Bits[1] = Num2Bits(253);
  bitmaskNum2Bits[1].in <== bitmask[1];

  for(var i = 0; i < 253; i++) {
    aggregateKeys.bitmask[i+6] <== bitmaskNum2Bits[1].out[i];
  }


  bitmaskNum2Bits[2] = Num2Bits(253);
  bitmaskNum2Bits[2].in <== bitmask[2];

  for(var i = 0; i < 253; i++) {
    aggregateKeys.bitmask[i+6+253] <== bitmaskNum2Bits[2].out[i];
  }

  component verify = CoreVerifyPubkeyG1(55, K);

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < K; k++) {
      verify.pubkey[j][k] <== aggregateKeys.out[j][k];
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        verify.signature[i][j][k] <== signature[i][j][k];
        verify.hash[i][j][k] <== hash[i][j][k];
      }
    }
  }

  component bits2Num[2];
  bits2Num[0] = Bits2Num(253);
  for(var i = 0; i < 253; i++) {
    bits2Num[0].in[i] <== hasher.out[252-i];
  }
  hashTreeRoot[1] <== bits2Num[0].out;

  bits2Num[1] = Bits2Num(3);

  for(var i = 0; i < 3; i++){
    bits2Num[1].in[i] <== hasher.out[255-i];
  }

  hashTreeRoot[0] <== bits2Num[1].out;
}
