pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "../../../node_modules/circomlib/circuits/bitify.circom";
include "../../../node_modules/circomlib/circuits/pedersen.circom";
include "validators_hash_tree_root.circom";
include "hash_tree_root_pedersen.circom";
include "bitmask_contains_only_bools.circom";
include "aggregate_bitmask.circom";
include "xnor.circom";
include "output_commitment.circom";

template AggregatePubKeys(N) {
  var J = 2;
  var K = 7;
  signal input points[N][J][K];
  signal input zero[N];

  signal input bitmask[N];

  signal input slashed[N];

  signal input activationEpoch[N];

  signal input exitEpoch[N];

  // should be after currentEpoch will be checked in final circuit
  signal input minExitEpoch;

  // should be before currentEpoch will be checked in final circuit
  signal input maxActivationEpoch;

  signal output output_commitment;

  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];

  component pedersenHashTreeRoot = HashTreeRootPedersen(N);
  component pedersen[N];

  component xnor[N];
  component and[2 * N];
  component notSlashed[N];

  for(var i = 0; i < N; i++) {
    activationEpochLessThan[i] = LessEqThan(64);

    activationEpochLessThan[i].in[0] <== activationEpoch[i];
    activationEpochLessThan[i].in[1] <==  maxActivationEpoch;

    exitEpochGreaterThan[i] = GreaterEqThan(64);

    exitEpochGreaterThan[i].in[0] <== exitEpoch[i];
    exitEpochGreaterThan[i].in[1] <== minExitEpoch;

    and[2 * i] = AND();
    and[2 * i].a <== exitEpochGreaterThan[i].out;
    and[2 * i].b <== activationEpochLessThan[i].out;

    notSlashed[i] = NOT();
    notSlashed[i].in <== slashed[i];

    and[2 * i + 1] = AND();
    and[2 * i + 1].a <== and[2 * i].out;
    and[2 * i + 1].b <== notSlashed[i].out;

    xnor[i] = XNOR();
    xnor[i].a <== bitmask[i];
    xnor[i].b <== and[2 * i + 1].out;

    xnor[i].out === 1;

    pedersen[i] = Pedersen(17);

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        pedersen[i].in[j * 7 + k] <== points[i][j][k];
      }
    }

    pedersen[i].in[14] <== activationEpoch[i];
    pedersen[i].in[15] <== exitEpoch[i];
    pedersen[i].in[16] <== slashed[i];

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
