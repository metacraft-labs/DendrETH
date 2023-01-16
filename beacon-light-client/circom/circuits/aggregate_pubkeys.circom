pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "../../../node_modules/circomlib/circuits/pedersen.circom";
include "validators_hash_tree_root.circom";
include "hash_tree_root_pedersen.circom";
include "bitmask_contains_only_bools.circom";
include "aggregate_bitmask.circom";

template AggregatePubKeys(N) {
  var J = 2;
  var K = 7;
  signal input points[N][J][K];
  signal input zero[N];

  signal input bitmask[N];

  signal input slashed[N];

  signal input activationEligibilityEpoch[N];
  signal input activationEpoch[N];

  signal input exitEpoch[N];

  signal input currentEpoch;

  signal output output_commitment;

  component activationEligibilityEpochLessThan[N];
  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];

  component pedersenHashTreeRoot = HashTreeRootPedersen(N);
  component pedersen[N];

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

    pedersen[i] = Pedersen(18);

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        pedersen[i].in[j * 7 + k] <== points[i][j][k];
      }
    }

    pedersen[i].in[14] <== activationEligibilityEpoch[i];
    pedersen[i].in[15] <== activationEpoch[i];
    pedersen[i].in[16] <== exitEpoch[i];
    pedersen[i].in[17] <== slashed[i];

    pedersenHashTreeRoot.leaves[i] <== pedersen[i].out[0] * zero[i];
  }

  component bitmaskContainsOnlyBools = BitmaskContainsOnlyBools(N);

  for(var i = 0; i < N; i++) {
    bitmaskContainsOnlyBools.bitmask[i] <== bitmask[i];
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

  component commitment = Pedersen(160);

  commitment.in[0] <== currentEpoch;
  commitment.in[1] <== pedersenHashTreeRoot.out;

  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      commitment.in[2 + j * 7 +k] <== aggregateKeys.out[j][k];
    }
  }

  for (var i = 0; i < 144; i++) {
    commitment.in[16 + i] <== 0;
  }

  output_commitment <== commitment.out[0];
}
