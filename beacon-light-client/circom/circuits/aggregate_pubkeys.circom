pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "validators_hash_tree_root.circom";
include "compress.circom";
include "bitmask_contains_only_bools.circom";
include "aggregate_bitmask.circom";
include "output_commitment.circom";

template AggregatePubKeys(N) {
  var J = 2;
  var K = 7;
  signal input points[N][J][K];
  signal input zero[N];

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
  component or[4*N];
  component not[N];

  for(var i = 0; i < N; i++) {
    activationEligibilityEpochLessThan[i] = LessThan(64);

    activationEligibilityEpochLessThan[i].in[0] <== activationEligibilityEpoch[i];
    activationEligibilityEpochLessThan[i].in[1] <== currentEpoch;

    not[i] = NOT();
    not[i].in <== bitmask[i];

    or[4 * i] = OR();
    or[4 * i].a <== not[i].out;
    or[4 * i].b <== activationEligibilityEpochLessThan[i].out;
    or[4 * i].out === 1;

    activationEpochLessThan[i] = LessThan(64);

    activationEpochLessThan[i].in[0] <== activationEpoch[i];
    activationEpochLessThan[i].in[1] <==  currentEpoch;

    or[4 * i + 1] = OR();
    or[4 * i + 1].a <== not[i].out;
    or[4 * i + 1].b <== activationEpochLessThan[i].out;
    or[4 * i + 1].out === 1;

    exitEpochGreaterThan[i] = GreaterThan(64);

    exitEpochGreaterThan[i].in[0] <== exitEpoch[i];
    exitEpochGreaterThan[i].in[1] <== currentEpoch;

    or[4 * i + 2] = OR();
    or[4 * i + 2].a <== not[i].out;
    or[4 * i + 2].b <== exitEpochGreaterThan[i].out;
    or[4 * i + 2].out === 1;

    slashedIsZero[i] = IsZero();
    slashedIsZero[i].in <== slashed[i];

    or[4 * i + 3] = OR();
    or[4 * i + 3].a <== not[i].out;
    or[4 * i + 3].b <== slashedIsZero[i].out;
    or[4 * i + 3].out === 1;

    compress[i] = Compress();

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        compress[i].point[j][k] <== points[i][j][k];
      }
    }

    for(var j = 0; j < 384; j++) {
      validatorsHashTreeRoot.pubkeys[i][j] <== compress[i].bits[j];
    }

    validatorsHashTreeRoot.zero[i] <== zero[i];
    validatorsHashTreeRoot.activationEligibilityEpoch[i] <== activationEligibilityEpoch[i];
    validatorsHashTreeRoot.activationEpoch[i] <== activationEpoch[i];
    validatorsHashTreeRoot.exitEpoch[i] <== exitEpoch[i];
    validatorsHashTreeRoot.slashed[i] <== slashed[i];

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

  component commitment = OutputCommitment();

  commitment.currentEpoch <== currentEpoch;
  commitment.participantsCount <== participantsCount;

  for(var i = 0; i < 256; i++) {
    commitment.hash[i] <== validatorsHashTreeRoot.out[i];
  }

  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      commitment.aggregatedKey[j][k] <== aggregateKeys.out[j][k];
    }
  }

  for (var i = 0;i < 6;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        commitment.negalfa1xbeta2[i][j][idx] <== 0;
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        commitment.gamma2[i][j][idx] <== 0;
        commitment.delta2[i][j][idx] <== 0;
        commitment.IC[i][j][idx] <== 0;
      }
    }
  }

  output_commitment <== commitment.out;
}
