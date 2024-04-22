pragma circom 2.1.5;

include "sync_commitee_hash_tree_root.circom";
include "compress.circom";
include "aggregate_bitmask.circom";
include "is_supermajority.circom";
include "is_valid_merkle_branch.circom";
include "hash_tree_root_beacon_header.circom";
include "compute_domain.circom";
include "compute_signing_root.circom";
include "hash_to_field.circom";
include "is_first.circom";
include "../../../vendor/circom-pairing/circuits/bls_signature.circom";
include "../../../vendor/circom-pairing/circuits/bn254/groth16.circom";

template LightClientRecursive(N, K) {
  var pubInpCount = 4;

  // BN254 facts
  var k = 6;

  // public inputs
  signal input originator[2];
  signal input nextHeaderHashNum[2];

  // private inputs
  signal input prevHeaderHashNum[2];

  // verification key
  signal input negalfa1xbeta2[6][2][k]; // e(-alfa1, beta2)
  signal input gamma2[2][2][k];
  signal input delta2[2][2][k];
  signal input IC[pubInpCount+1][2][k];

  // proof
  signal input negpa[2][k];
  signal input pb[2][2][k];
  signal input pc[2][k];

  signal input slot[256];
  signal input proposer_index[256];
  signal input parent_root[256];
  signal input state_root[256];
  signal input body_root[256];

  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal input points[N][2][K];
  signal input aggregatedKey[384];
  signal input branch[5][256];

  signal input bitmask[N];
  signal input signature[2][2][K];

  var prevHeaderHash[256];
  var nextHeaderHash[256];

  signal num2bits1[253] <== Num2Bits(253)(prevHeaderHashNum[0]);

  signal num2bits2[3] <== Num2Bits(3)(prevHeaderHashNum[1]);

  for(var i = 0; i < 253; i++) {
    prevHeaderHash[i] = num2bits1[252 - i];
  }

  for(var i = 253; i < 256; i++) {
    prevHeaderHash[i] = num2bits2[255 - i];
  }

  signal num2bits3[253] <== Num2Bits(253)(nextHeaderHashNum[0]);

  for(var i = 0; i < 253; i++) {
    nextHeaderHash[i] = num2bits3[252 - i];
  }

  signal num2bits4[3] <== Num2Bits(3)(nextHeaderHashNum[1]);

  for(var i = 253; i < 256; i++) {
    nextHeaderHash[i] = num2bits4[255 - i];
  }

  IsSuperMajority(N)(bitmask);

  signal hash_tree_root_beacon[256] <== HashTreeRootBeaconHeader()(slot,proposer_index,parent_root,state_root,body_root);

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon[i] === prevHeaderHash[i];
  }

  signal computeDomain[256] <== ComputeDomain()(fork_version,GENESIS_VALIDATORS_ROOT,DOMAIN_SYNC_COMMITTEE);

  signal computeSigningRoot[256] <== ComputeSigningRoot()(nextHeaderHash,computeDomain);

  signal hashToField[2][2][K] <== HashToField(K)(computeSigningRoot);

  component hasher = SyncCommiteeHashTreeRoot(N);
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

  IsValidMerkleBranch(5)(branch,hasher.out,state_root,55);

  signal aggregateKeys[2][K] <== AggregateKeysBitmask(N,K)(points, bitmask);

  CoreVerifyPubkeyG1(55, K)(aggregateKeys,signature,hashToField);

  // check recursive snark
  component groth16Verifier = verifyProof(pubInpCount);
  for (var i = 0;i < 6;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.negalfa1xbeta2[i][j][idx] <== negalfa1xbeta2[i][j][idx];
          }
      }
  }
  for (var i = 0;i < 2;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.gamma2[i][j][idx] <== gamma2[i][j][idx];
              groth16Verifier.delta2[i][j][idx] <== delta2[i][j][idx];
              groth16Verifier.pb[i][j][idx] <== pb[i][j][idx];
          }
      }
  }
  for (var i = 0;i < pubInpCount + 1;i++) {
      for (var j = 0;j < 2;j++) {
          for (var idx = 0;idx < k;idx++) {
              groth16Verifier.IC[i][j][idx] <== IC[i][j][idx];
          }
      }
  }
  for (var i = 0;i < 2;i++) {
      for (var idx = 0;idx < k;idx++) {
          groth16Verifier.negpa[i][idx] <== negpa[i][idx];
          groth16Verifier.pc[i][idx] <== pc[i][idx];
      }
  }

  groth16Verifier.pubInput[0] <== originator[0];
  groth16Verifier.pubInput[1] <== originator[1];
  groth16Verifier.pubInput[2] <== prevHeaderHashNum[0];
  groth16Verifier.pubInput[3] <== prevHeaderHashNum[1];

  signal isFirst <== IsFirst()(originator,prevHeaderHashNum);

  component firstORcorrect = OR();
  firstORcorrect.a <== isFirst;
  firstORcorrect.b <== groth16Verifier.out;

  firstORcorrect.out === 1;
}
