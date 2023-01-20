pragma circom 2.0.3;

include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";
include "hash_tree_root_pedersen.circom";
include "hash_tree_root.circom";

template CommitmentMapperAggregator(N) {
  var pubInpCount = 1;
  // BN254 facts
  var k = 6;

  signal input shaHashes[N][256];
  signal input pedersenHashes[N];

  // verification key
  signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input gamma2[2][2][k];
  signal input delta2[2][2][k];
  signal input IC[pubInpCount+1][2][k];

  signal input zeroOnFirst;

  // proof
  signal input negpa[N][2][k];
  signal input pb[N][2][2][k];
  signal input pc[N][2][k];

  signal output output_commitment;

  component hashTreeRootPedersen = HashTreeRootPedersen(N);
    component hashTreeRoot = HashTreeRoot(N);

  for(var i = 0; i < N; i++) {
    hashTreeRootPedersen.leaves[i] <== pedersenHashes[i];
    for (var j = 0; j < 256; j++) {
      hashTreeRoot.leaves[i][j] <== shaHashes[i][j];
    }
  }

  component commitment = Pedersen(401);

  for(var i = 0; i < 256; i++) {
    commitment.in[i] <== hashTreeRoot.out[i];
  }

  commitment.in[256] <== hashTreeRoot.out;

  for (var i = 0; i < 144; i++) {
    commitment.in[257 + i] <== 0;
  }

  for(var i = 0; i < 6; i++) {
    for(var j = 0; j<2;j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[257 + i * 12 + j * 6 + k] <== zeroOnFirst * negalfa1xbeta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[329 + i * 12 + j * 6 + k] <== zeroOnFirst * gamma2[i][j][idx];
        commitment.in[353 + i * 12 + j * 6 + k] <== zeroOnFirst * delta2[i][j][idx];
      }
    }
  }

  for(var i = 0; i < pubInpCount + 1; i++) {
    for(var j = 0; j < 2; j++) {
      for(var idx = 0; idx < k; idx++) {
        commitment.in[377 + i * 12 + j * 6 + k] <== zeroOnFirst * IC[i][j][idx];
      }
    }
  }

  output_commitment <== commitment.out[0];

  component groth16Verifier[N];
  component prevCommitments[N];

  for(var index = 0; index < N; index++) {
    groth16Verifier[index] = verifyProof(pubInpCount);

    for (var i = 0;i < 6;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].negalfa1xbeta2[i][j][idx] <== zeroOnFirst * negalfa1xbeta2[i][j][idx] + (1 - zeroOnFirst) * firstNegalfa1xbeta2[i][j][idx];
        }
      }
    }

    for (var i = 0;i < 2;i++) {
      for (var j = 0;j < 2;j++) {
        for (var idx = 0;idx < k;idx++) {
          groth16Verifier[index].gamma2[i][j][idx] <== zeroOnFirst * gamma2[i][j][idx] + (1 - zeroOnFirst) * firstGama2[i][j][idx];
          groth16Verifier[index].delta2[i][j][idx] <== zeroOnFirst * delta2[i][j][idx] + (1 - zeroOnFirst) * firstDelta2[i][j][idx];
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

    prevCommitments[index] = Pedersen(401);

    for(var i = 0; i < 256; i++) {
      prevCommitments[index].in[i] <== shaHashes[index][i];
    }

    prevCommitments[index].in[256] <== pedersenHashes[index];

    for(var i = 0; i < 6; i++) {
      for(var j = 0; j<2;j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[257 + i * 12 + j * 6 + k] <== zeroOnFirst * negalfa1xbeta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < 2; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[329 + i * 12 + j * 6 + k] <== zeroOnFirst * gamma2[i][j][idx];
          prevCommitments[index].in[353 + i * 12 + j * 6 + k] <== zeroOnFirst * delta2[i][j][idx];
        }
      }
    }

    for(var i = 0; i < pubInpCount + 1; i++) {
      for(var j = 0; j < 2; j++) {
        for(var idx = 0; idx < k; idx++) {
          prevCommitments[index].in[377 + i * 12 + j * 6 + k] <== zeroOnFirst * IC[i][j][idx];
        }
      }
    }

    output_commitment <== commitment.out[0];
  }
}
