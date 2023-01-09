pragma circom 2.0.3;

include "is_valid_merkle_branch_pedersen.circom";
include "aggregate_bitmask.circom";
include "../../../node_modules/circomlib/circuits/pedersen.circom";

template AggregateSubtract(N) {
  var J = 2;
  var K = 7;
  signal input points[N][J][K];
  signal input slashed[N];

  signal input activationEpoch[N];

  signal input exitEpoch[N];

  signal input state_root;

  signal input branches[N][41];
  signal input indexes[N];

  // should be after currentEpoch will be checked in final circuit
  signal input minExitEpoch;

  // should be before currentEpoch will be checked in final circuit
  signal input maxActivationEpoch;

  signal output output_commitment;

  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];


  component pedersen[N];

  for(var i = 0; i < N; i++) {
    activationEpochLessThan[i] = LessEqThan(64);

    activationEpochLessThan[i].in[0] <== activationEpoch[i];
    activationEpochLessThan[i].in[1] <==  maxActivationEpoch;

    activationEpochLessThan[i].out === 1;

    exitEpochGreaterThan[i] = GreaterEqThan(64);

    exitEpochGreaterThan[i].in[0] <== exitEpoch[i];
    exitEpochGreaterThan[i].in[1] <== minExitEpoch;

    exitEpochGreaterThan[i].out === 1;

    slashed[i] === 0;

    pedersen[i] = Pedersen(17);

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        pedersen[i].in[j * 7 + k] <== points[i][j][k];
      }
    }

    pedersen[i].in[14] <== activationEpoch[i];
    pedersen[i].in[15] <== exitEpoch[i];
    pedersen[i].in[16] <== slashed[i];
  }

  component isValidMerkleBranch[N];
  component indexLessThan[N - 1];

  for(var i = 0; i < N; i++) {
    isValidMerkleBranch[i] = IsValidMerkleBranchPedersen(41);

    for(var j = 0; j < 41; j++) {
      isValidMerkleBranch[i].branch[j] <== branches[i][j];
    }

    isValidMerkleBranch[i].leaf <== pedersen[i].out[0];
    isValidMerkleBranch[i].root <== state_root;
    isValidMerkleBranch[i].index <== indexes[i];

    isValidMerkleBranch[i].out === 1;

    // check order
    if (i < N - 1) {
      indexLessThan[i] = LessThan(42);
      indexLessThan[i].in[0] <== indexes[i];
      indexLessThan[i].in[1] <== indexes[i + 1];
      indexLessThan[i].out === 1;
    }
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
    aggregateKeys.bitmask[i] <== 1;
  }

  component hasher = Pedersen(161);

  hasher.in[0] <== minExitEpoch;
  hasher.in[1] <== maxActivationEpoch;

  for (var j = 0; j < J; j++) {
    for (var k = 0; k < K; k++) {
      hasher.in[2 + j * K + k] <== aggregateKeys.out[j][k];
    }
  }

  hasher.in[16] <== state_root;

  for(var i = 0; i < 144; i++) {
    hasher.in[17 + i] <== 0;
  }

  output_commitment <== hasher.out[0];
}
