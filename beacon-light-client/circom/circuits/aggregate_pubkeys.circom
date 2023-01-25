pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "../../../node_modules/circomlib/circuits/pedersen.circom";
include "validators_hash_tree_root.circom";
include "hash_tree_root_pedersen.circom";
include "bitmask_contains_only_bools.circom";
include "aggregate_bitmask.circom";
include "output_commitment.circom";

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

  // should be after currentEpoch will be checked in final circuit
  signal input minExitEpoch;

  // should be before currentEpoch will be checked in final circuit
  signal input maxActivationEpoch;

  signal output output_commitment;

  component activationEligibilityEpochLessThan[N];
  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];

  component pedersenHashTreeRoot = HashTreeRootPedersen(N);
  component pedersen[N];

  component activationEligibilityEpochBits[N];
  component activationEpochBits[N];
  component exitEpochBits[N];
  component not[N];
  component xor[4*N];
  component notSlashed[N];

  for(var i = 0; i < N; i++) {
    activationEligibilityEpochLessThan[i] = LessEqThan(64);

    activationEligibilityEpochLessThan[i].in[0] <== activationEligibilityEpoch[i];
    activationEligibilityEpochLessThan[i].in[1] <== maxActivationEpoch;

    not[i] = NOT();
    not[i].in <== bitmask[i];

    xor[4 * i] = XOR();
    xor[4 * i].a <== not[i].out;
    xor[4 * i].b <== activationEligibilityEpochLessThan[i].out;
    xor[4 * i].out === 1;

    activationEpochLessThan[i] = LessEqThan(64);

    activationEpochLessThan[i].in[0] <== activationEpoch[i];
    activationEpochLessThan[i].in[1] <==  maxActivationEpoch;

    xor[4 * i + 1] = XOR();
    xor[4 * i + 1].a <== not[i].out;
    xor[4 * i + 1].b <== activationEpochLessThan[i].out;
    xor[4 * i + 1].out === 1;

    exitEpochGreaterThan[i] = GreaterEqThan(64);

    exitEpochGreaterThan[i].in[0] <== exitEpoch[i];
    exitEpochGreaterThan[i].in[1] <== minExitEpoch;

    xor[4 * i + 2] = XOR();
    xor[4 * i + 2].a <== not[i].out;
    xor[4 * i + 2].b <== exitEpochGreaterThan[i].out;
    xor[4 * i + 2].out === 1;

    notSlashed[i] = NOT();
    notSlashed[i].in <== slashed[i];

    xor[4 * i + 3] = XOR();
    xor[4 * i + 3].a <== notSlashed[i].out;
    xor[4 * i + 3].b <== not[i].out;
    xor[4 * i + 3].out === 1;

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

  component commitment = OutputCommitment();

  commitment.maxActivationEpoch <== maxActivationEpoch;
  commitment.minExitEpoch <== minExitEpoch;
  commitment.hash <== pedersenHashTreeRoot.out;

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
