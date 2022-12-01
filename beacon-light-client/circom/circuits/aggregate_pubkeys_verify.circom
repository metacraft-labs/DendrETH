pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/curve.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "output_commitment.circom";
include "hash_tree_root.circom";
include "aggregate_bitmask.circom";

template AggregatePubKeysVerify(N) {
  var J = 2;
  var K = 7;
  var pubInpCount = 1;
  // BN254 facts
  var k = 6;

  // verification key
  signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input gamma2[2][2][k];
  signal input delta2[2][2][k];
  signal input IC[pubInpCount+1][2][k];

  // prev verification key
  signal input prevNegalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input prevGamma2[2][2][k];
  signal input prevDelta2[2][2][k];
  signal input prevIC[pubInpCount+1][2][k];

  // proof
  signal input negpa[N][2][k];
  signal input pb[N][2][2][k];
  signal input pc[N][2][k];

  signal input points[N][J][K];
  signal input bitmask[N];

  signal input hashes[N][256];

  // TODO: currentEpoch to be array and to compare every value based on the bitmask so we can have epoch independent proofs for the zeros
  signal input currentEpoch;
  signal input participantsCount[N];

  signal output output_commitment;

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

  var participantsSum = 0;
  for(var i = 0; i < N; i++) {
    participantsSum += participantsCount[i];
  }

  component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== hashes[i][j];
    }
  }

  component commitment = OutputCommitment();

  commitment.currentEpoch <== currentEpoch;
  commitment.participantsCount <== participantsSum;

  for(var i = 0; i < 256; i++) {
    commitment.hash[i] <== hashTreeRoot.out[i];
  }

  for(var j = 0; j < J; j++) {
    for(var k = 0; k < K; k++) {
      commitment.aggregatedKey[j][k] <== aggregateKeys.out[j][k];
    }
  }

  for(var i = 0; i < 6; i++) {
    for(var j = 0; j<2;j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.gamma2[i][j][idx] <== gamma2[i][j][idx];
        commitment.delta2[i][j][idx] <== delta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < pubInpCount + 1; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.IC[i][j][idx] <== IC[i][j][idx];
      }
    }
  }

  output_commitment <== commitment.out;

  // check recursive snark
  component groth16Verifier[N];
  component prevCommitments[N];

  for(var index = 0; index < N; index++) {
    groth16Verifier[index] = verifyProof(pubInpCount);

    for (var i = 0;i < 6;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
        }
      }
    }

    for (var i = 0;i < 2;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].gamma2[i][j][idx] <== gamma2[i][j][idx];
          groth16Verifier[index].delta2[i][j][idx] <== delta2[i][j][idx];
          groth16Verifier[index].pb[i][j][idx] <== pb[index][i][j][idx];
        }
      }
    }

    for (var i = 0;i < pubInpCount + 1;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].IC[i][j][idx] <== IC[i][j][idx];
        }
      }
    }

    for (var i = 0;i < 2;i++) {
      for (var idx = 0;idx < k;idx++) {
        groth16Verifier[index].negpa[i][idx] <== negpa[index][i][idx];
        groth16Verifier[index].pc[i][idx] <== pc[index][i][idx];
      }
    }

    prevCommitments[index] = OutputCommitment();

    prevCommitments[index].currentEpoch <== currentEpoch;
    prevCommitments[index].participantsCount <== participantsCount[index];

    for(var i = 0; i < 256; i++) {
      prevCommitments[index].hash[i] <== hashes[index][i];
    }

    for(var j = 0; j < J; j++) {
      for(var k = 0; k < K; k++) {
        prevCommitments[index].aggregatedKey[j][k] <== points[index][j][k];
      }
    }

    for(var i = 0; i < 6; i++) {
      for(var j = 0; j<2;j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].negalfa1xbeta2[i][j][idx] <== prevNegalfa1xbeta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < 2; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].gamma2[i][j][idx] <== prevGamma2[i][j][idx];
          prevCommitments[index].delta2[i][j][idx] <== prevDelta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < pubInpCount + 1; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].IC[i][j][idx] <== prevIC[i][j][idx];
        }
      }
    }

    groth16Verifier[index].pubInput[0] <== prevCommitments[index].out;
    groth16Verifier[index].out === 1;
  }
}
