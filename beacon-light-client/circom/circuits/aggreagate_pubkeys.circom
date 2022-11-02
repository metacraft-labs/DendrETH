pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../node_modules/circomlib/circuits/poseidon.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "validators_hash_tree_root.circom";
include "compress.circom";
include "bitmask_contains_only_bools.circom";
include "aggregate_bitmask.circom";

template AggregatePubKeys(N) {
  var J = 2;
  var K = 7;
  signal input points[N][J][K];

  signal input withdrawCredentials[N][256];

  signal input effectiveBalance[N][256];
  signal input slashed[N];

  signal input activationEligibilityEpoch[N];
  signal input activationEpoch[N];

  signal input exitEpoch[N];
  signal input withdrawableEpoch[N][256];

  signal input bitmask[N];

  signal input currentEpoch;

  signal output output_commitment;

  component activationEligibilityEpochLessThan[N];
  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];
  component validatorsHashTreeRoot = ValidatorsHashTreeRoot(N);
  component compress[N];
  component activationEligibilityEpochBits[N];
  component activationEpochBits[N];
  component exitEpochBits[N];
  component slashedIsZero[N];


  for(var i = 0; i < N; i++) {
    activationEligibilityEpochLessThan[i] = LessThan(64);

    activationEligibilityEpochLessThan[i].in[0] <== activationEligibilityEpoch[i];
    activationEligibilityEpochLessThan[i].in[1] <== currentEpoch;
    activationEligibilityEpochLessThan[i].out === 1;

    activationEpochLessThan[i] = LessThan(64);

    activationEpochLessThan[i].in[0] <== activationEpoch[i];
    activationEpochLessThan[i].in[1] <==  currentEpoch;
    activationEpochLessThan[i].out === 1;

    exitEpochGreaterThan[i] = GreaterThan(64);

    exitEpochGreaterThan[i].in[0] <== exitEpoch[i];
    exitEpochGreaterThan[i].in[1] <== currentEpoch;
    exitEpochGreaterThan[i].out === 1;

    slashedIsZero[i] = IsZero();
    slashedIsZero[i].in <== slashed[i];

    slashedIsZero[i].out === 1;

    compress[i] = Compress();

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        compress[i].point[j][k] <== points[i][j][k];
      }
    }

    for(var j = 0; j < 384; j++) {
      validatorsHashTreeRoot.pubkeys[i][j] <== compress[i].bits[j];
    }

    activationEligibilityEpochBits[i] = Num2Bits(64);
    activationEligibilityEpochBits[i].in <== activationEligibilityEpoch[i];

    for(var j = 0; j < 64; j++) {
      validatorsHashTreeRoot.activationEligibilityEpoch[i][j] <== activationEligibilityEpochBits[i].out[63 - j];
    }

    for(var j = 64; j < 256; j++) {
      validatorsHashTreeRoot.activationEligibilityEpoch[i][j] <== 0;
    }

    activationEpochBits[i] = Num2Bits(64);
    activationEpochBits[i].in <== activationEpoch[i];

    for(var j = 0; j < 64; j++) {
      validatorsHashTreeRoot.activationEpoch[i][j] <== activationEpochBits[i].out[63 - j];
    }


    for(var j = 64; j < 256; j++) {
      validatorsHashTreeRoot.activationEpoch[i][j] <== 0;
    }

    exitEpochBits[i] = Num2Bits(64);
    exitEpochBits[i].in <== exitEpoch[i];

    for(var j = 0; j < 64; j++) {
      validatorsHashTreeRoot.exitEpoch[i][j] <== exitEpochBits[i].out[63 - j];
    }

    for(var j = 64; j < 256; j++) {
      validatorsHashTreeRoot.exitEpoch[i][j] <== 0;
    }

    validatorsHashTreeRoot.slashed[i][0] <== slashed[i];

    for(var j = 1; j < 256; j++) {
      validatorsHashTreeRoot.slashed[i][j] <== 0;
    }

    for(var j = 0; j < 256; j++) {
      validatorsHashTreeRoot.withdrawCredentials[i][j] <== withdrawCredentials[i][j];
      validatorsHashTreeRoot.effectiveBalance[i][j] <== effectiveBalance[i][j];
      validatorsHashTreeRoot.withdrawableEpoch[i][j] <== withdrawableEpoch[i][j];
    }
  }

  component bitmaskContainsOnlyBools = BitmaskContainsOnlyBools(N);

  for(var i = 0; i < N; i++) {
    bitmaskContainsOnlyBools.bitmask[i] <== bitmask[i];
  }

  var participantsCount = 0;

  for(var i = 0; i < N; i++) {
    participantsCount += bitmask[i];
  }

  component aggregateKeys = AggregateKeysBitmask(N);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        aggregateKeys.points[i][j][k] <== points[i][j][k];
      }
    }
  }

  for(var i = 0; i < N; i++) {
    aggregateKeys.bitmask[i] <== bitmask[i];
  }

  component poseidon = Poseidon(16);

  poseidon.inputs[0] <== currentEpoch;
  poseidon.inputs[1] <== participantsCount;

  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      poseidon.inputs[2 + j * 7 + k] <== aggregateKeys.out[j][k];
    }
  }

  output_commitment <== poseidon.out;
}
