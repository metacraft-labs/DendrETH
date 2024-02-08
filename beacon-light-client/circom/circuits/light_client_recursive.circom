pragma circom 2.1.5;

include "hash_verifier_poseidon.circom";
include "sync_committee_historic_participation.circom"
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
  var pubInpCount = 1;
  var PERIODS = 1024;

  // BN254 facts
  var k = 6;

  // public inputs
  signal input originator[2];
  signal input nextHeaderHashNum[2];
  signal input syncCommitteeHistoricParticipation[PERIODS];

  // private inputs
  signal input prevHeaderHashNum[2];
  signal input syncCommitteeHistoricParticipationIndex;

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

  // Exposed as public via domain
  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal input points[N][2][K];
  signal input aggregatedKey[384];
  signal input branch[5][256];

  signal input bitmask[N];
  signal input signature[2][2][K];

  signal output out; // Poseidon Hash of inputs & verification key

  var prevHeaderHash[256];
  var nextHeaderHash[256];

  component num2bits1 = Num2Bits(253);
  num2bits1.in <== prevHeaderHashNum[0];

  component num2bits2 = Num2Bits(3);
  num2bits2.in <== prevHeaderHashNum[1];

  for(var i = 0; i < 253; i++) {
    prevHeaderHash[i] = num2bits1.out[252 - i];
  }

  for(var i = 253; i < 256; i++) {
    prevHeaderHash[i] = num2bits2.out[255 - i];
  }

  component num2bits3 = Num2Bits(253);
  num2bits3.in <== nextHeaderHashNum[0];

  for(var i = 0; i < 253; i++) {
    nextHeaderHash[i] = num2bits3.out[252 - i];
  }

  component num2bits4 = Num2Bits(3);
  num2bits4.in <== nextHeaderHashNum[1];

  for(var i = 253; i < 256; i++) {
    nextHeaderHash[i] = num2bits4.out[255 - i];
  }

  component isSuperMajority = IsSuperMajority(N);

  for(var i = 0; i < N; i++) {
    isSuperMajority.bitmask[i] <== bitmask[i];
  }

  component hash_tree_root_beacon = HashTreeRootBeaconHeader();

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.slot[i] <== slot[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.proposer_index[i] <== proposer_index[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.parent_root[i] <== parent_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.state_root[i] <== state_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.body_root[i] <== body_root[i];
  }

  for(var i = 0; i < 256; i++) {
    hash_tree_root_beacon.out[i] === prevHeaderHash[i];
  }

  component computeDomain = ComputeDomain();

  computeDomain.fork_version <== fork_version;
  computeDomain.GENESIS_VALIDATORS_ROOT <== GENESIS_VALIDATORS_ROOT;
  computeDomain.DOMAIN_SYNC_COMMITTEE <== DOMAIN_SYNC_COMMITTEE;

  component computeSigningRoot = ComputeSigningRoot();

  computeSigningRoot.domain <== computeDomain.domain;

  for(var i = 0; i < 256; i++) {
    computeSigningRoot.headerHash[i] <== nextHeaderHash[i];
  }

  component hashToField = HashToField(K);

  for(var i = 0; i < 256; i++) {
    hashToField.in[i] <== computeSigningRoot.signing_root[i];
  }

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

  component isValidMerkleBranch = IsValidMerkleBranch(5);

  for(var i = 0; i < 5; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranch.branch[i][j] <== branch[i][j];
    }
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranch.leaf[i] <== hasher.out[i];
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranch.root[i] <== state_root[i];
  }

  isValidMerkleBranch.index <== 55;

  component aggregateKeys = AggregateKeysBitmask(N,K);

  for(var i = 0; i < N; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        aggregateKeys.points[i][j][k] <== points[i][j][k];
      }
    }
  }

  for(var i = 0; i < N; i++) {
    aggregateKeys.bitmask[i] <== bitmask[i];
  }

  component verify = CoreVerifyPubkeyG1(55, K);

  for(var j = 0; j < 2; j++) {
    for(var k = 0; k < K; k++) {
      verify.pubkey[j][k] <== aggregateKeys.out[j][k];
    }
  }

  for(var i = 0; i < 2; i++) {
    for(var j = 0; j < 2; j++) {
      for(var k = 0; k < K; k++) {
        verify.signature[i][j][k] <== signature[i][j][k];
        verify.hash[i][j][k] <== hashToField.out[i][j][k];
      }
    }
  }

  // check recursive snark
  component groth16Verifier = verifyProof(1);
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

  component prevHistoricParticipationRateHashTreeRoot = HashTreeRootPoseidon(PERIODS) (
    syncCommitteeHistoricParticipation
  );

  signal prevSyncCommitteeHistoricParticipationIndex <== syncCommitteeHistoricParticipationIndex - 1;

  signal prevVerifierCommitment <== VerifierPoseidon(pubInpCount, k)(
    originator, prevHeaderHashNum, negalfa1xbeta2,
    gamma2, delta2, IC,
    historicParticipationRateHashTreeRoot, prevSyncCommitteeHistoricParticipationIndex, computeDomain.domain
  );

  component updateSyncCommitteeHistoricParticipation = UpdateSyncCommitteeHistoricParticipation(512,PERIODS) (
    syncCommitteeHistoricParticipation, syncCommitteeHistoricParticipationIndex, bitmask
  );

  signal curSyncCommitteeHistoricParticipation <== updateSyncCommitteeHistoricParticipation.out;

  component curHistoricParticipationRateHashTreeRoot = HashTreeRootPoseidon(PERIODS) (
    curSyncCommitteeHistoricParticipation
  );

  groth16Verifier.pubInput[0] <== prevVerifierCommitment;
 
  component isFirst = IsFirst();

  isFirst.firstHash[0] <== originator[0];
  isFirst.firstHash[1] <== originator[1];
  isFirst.secondHash[0] <== prevHeaderHashNum[0];
  isFirst.secondHash[1] <== prevHeaderHashNum[1];

  component firstORcorrect = OR();
  firstORcorrect.a <== isFirst.out;
  firstORcorrect.b <== groth16Verifier.out;

  firstORcorrect.out === 1;

  component verifierPoseidon = VerifierPoseidon(pubInpCount, k) (
    originator, nextHeaderHashNum, negalfa1xbeta2,
    gamma2, delta2, IC,
    curHistoricParticipationRateHashTreeRoot, syncCommitteeHistoricParticipationIndex, compute_domain.domain
  );

  prevVerifierCommitment === verifierPoseidon.out;
  out <== verifierPoseidon.out;
}
