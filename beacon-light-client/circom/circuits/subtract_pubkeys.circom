pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../node_modules/circomlib/circuits/mimcsponge.circom";
include "validator_hash_tree_root.circom";
include "compress.circom";
include "is_valid_merkle_branch.circom";

template SubtractPubkeys(N) {
  var J = 2;
  var K = 7;
  signal input branches[N][41][256];

  signal input aggregatedKey[J][K];

  signal input points[N][J][K];
  signal input withdrawCredentials[N][256];

  signal input effectiveBalance[N][256];
  signal input slashed[N];

  signal input activationEligibilityEpoch[N];
  signal input activationEpoch[N];

  signal input exitEpoch[N];
  signal input withdrawableEpoch[N][256];

  signal input indexes[N];

  signal input currentEpoch;
  signal input state_root[256];

  signal output output_commitment;

  component activationEligibilityEpochLessThan[N];
  component activationEpochLessThan[N];
  component exitEpochGreaterThan[N];
  component slashedIsZero[N];

  component validatorHashTreeRoot[N];
  component compress[N];

  component activationEligibilityEpochBits[N];
  component activationEpochBits[N];
  component exitEpochBits[N];

  component isValidMerkleBranch[N];

  component indexLessThan[N - 1];

  for (var i = 0; i < N; i++) {
    // active validator checks
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

    // validator hashTreeRootCalculation
    compress[i] = Compress();

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        compress[i].point[j][k] <== points[i][j][k];
      }
    }

    validatorHashTreeRoot[i] = ValidatorHashTreeRoot();

    for(var j = 0; j < 384; j++) {
      validatorHashTreeRoot[i].pubkey[j] <== compress[i].bits[j];
    }

    validatorHashTreeRoot[i].activationEligibilityEpoch <== activationEligibilityEpoch[i];
    validatorHashTreeRoot[i].activationEpoch <== activationEpoch[i];
    validatorHashTreeRoot[i].exitEpoch <== exitEpoch[i];
    validatorHashTreeRoot[i].slashed <== slashed[i];

    for(var j = 0; j < 256; j++) {
      validatorHashTreeRoot[i].withdrawCredentials[j] <== withdrawCredentials[i][j];
      validatorHashTreeRoot[i].effectiveBalance[j] <== effectiveBalance[i][j];
      validatorHashTreeRoot[i].withdrawableEpoch[j] <== withdrawableEpoch[i][j];
    }

    // is valid merkle branch checks
    isValidMerkleBranch[i] = IsValidMerkleBranch(41);

    for(var j = 0; j < 41; j++) {
      for(var k = 0; k < 256; k++) {
        isValidMerkleBranch[i].branch[j][k] <== branches[i][j][k];
      }
    }

    for(var j = 0; j < 256; j++) {
      isValidMerkleBranch[i].leaf[j] <== validatorHashTreeRoot[i].out[j];
    }

    for(var j = 0; j < 256; j++) {
      isValidMerkleBranch[i].root[j] <== state_root[j];
    }

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

  component sub[N];
  sub[0] = EllipticCurveSubtract(55, 7, 0, 4, [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425]);
  for (var j = 0; j < J; j++) {
    for (var k = 0; k < K; k++) {
      sub[0].a[j][k] <== aggregatedKey[j][k];
      sub[0].b[j][k] <== points[0][j][k];
    }
  }
  sub[0].aIsInfinity <== 0;
  sub[0].bIsInfinity <== 0;

  // TODO subtract
  for (var i = 1; i < N; i++) {
    sub[i] = EllipticCurveSubtract(55, 7, 0, 4, [35747322042231467, 36025922209447795, 1084959616957103, 7925923977987733, 16551456537884751, 23443114579904617, 1829881462546425]);
    for (var j = 0; j < J; j++) {
      for (var k = 0; k < K; k++) {
        sub[i].a[j][k] <== sub[i - 1].out[j][k];
        sub[i].b[j][k] <== points[i][j][k];
      }
    }

    sub[i].aIsInfinity <== sub[i - 1].isInfinity;
    sub[i].bIsInfinity <== 0;
  }

  component hasher = MiMCSponge(429, 220, 1);
  hasher.k <== 123;

  hasher.ins[0] <== currentEpoch;

  for(var i = 0; i < 256; i++) {
    hasher.ins[1 + i] <== state_root[i];
  }

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < 7; k++) {
      hasher.ins[257 + j * 7 + k] <== aggregatedKey[j][k];
      hasher.ins[271 + j * 7 + k] <== sub[N-1].out[j][k];
    }
  }

  var vkCounter = 285;

  for (var i = 0;i < 6;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.ins[vkCounter] <== 0;
        vkCounter++;
      }
    }
  }

  for (var i = 0;i < 2;i++) {
    for (var j = 0;j < 2;j++) {
      for (var idx = 0;idx < 6;idx++) {
        hasher.ins[vkCounter] <== 0;
        vkCounter++;
        hasher.ins[vkCounter] <== 0;
        vkCounter++;
        hasher.ins[vkCounter] <== 0;
        vkCounter++;
      }
    }
  }

  output_commitment <== hasher.outs[0];
}
