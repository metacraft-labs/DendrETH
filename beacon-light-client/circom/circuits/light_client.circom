pragma circom 2.0.3;

include "sync_commitee_hash_tree_root.circom";
include "compress.circom";
include "aggregate_bitmask.circom";
include "is_supermajority.circom";
include "bitmask_contains_only_bools.circom";
include "is_valid_merkle_branch.circom";
include "compute_domain.circom";
include "compute_signing_root.circom";
include "hash_to_field.circom";
include "hash_tree_root_beacon_header.circom";
include "ssz_num.circom";
include "../../../vendor/circom-pairing/circuits/bls_signature.circom";

template LightClient(N) {
  var K = 7;
  signal input prevHeaderHash[256];
  signal input nextHeaderHash[256];

  signal input prevHeaderStateRoot[256];
  signal input prevHeaderStateRootBranch[3][256];

  signal input prevHeaderFinalizedSlot;
  signal input prevHeaderFinalizedSlotBranch[9][256];

  signal input nextHeaderSlot;
  signal input nextHeaderSlotBranch[3][256];

  signal input signatureSlot;

  signal input signatureSlotSyncCommitteePeriod;
  signal input finalizedHeaderSlotSyncCommitteePeriod;

  signal input finalizedHeaderRoot[256];
  signal input finalizedHeaderBranch[9][256];

  signal input execution_state_root[256];
  signal input execution_state_root_branch[11][256];

  // Should be harcoded
  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal input points[N][2][K];
  signal input aggregatedKey[384];
  signal input syncCommitteeBranch[5][256];

  signal input bitmask[N];
  signal input signature[2][2][K];

  signal output output_commitment[2];

  component isValidMerkleBranchPrevHeaderStateRoot = IsValidMerkleBranch(3);

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchPrevHeaderStateRoot.leaf[i] <== prevHeaderStateRoot[i];
    isValidMerkleBranchPrevHeaderStateRoot.root[i] <== prevHeaderHash[i];
  }

  for(var i = 0; i < 3; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchPrevHeaderStateRoot.branch[i][j] <== prevHeaderStateRootBranch[i][j];
    }
  }

  isValidMerkleBranchPrevHeaderStateRoot.index <== 11;
  isValidMerkleBranchPrevHeaderStateRoot.out === 1;

  component signatureSlotGreaterThanNext = GreaterThan(64);
  signatureSlotGreaterThanNext.in[0] <== signatureSlot;
  signatureSlotGreaterThanNext.in[1] <== nextHeaderSlot;
  signatureSlotGreaterThanNext.out === 1;

  component nextHeaderSlotGreaterThanPrevFinalized = GreaterThan(64);
  nextHeaderSlotGreaterThanPrevFinalized.in[0] <== nextHeaderSlot;
  nextHeaderSlotGreaterThanPrevFinalized.in[1] <== prevHeaderFinalizedSlot;
  nextHeaderSlotGreaterThanPrevFinalized.out === 1;

  component signatureSlotSyncCommitteePeriodLessThan = LessEqThan(64);
  signatureSlotSyncCommitteePeriodLessThan.in[0] <== signatureSlotSyncCommitteePeriod * 8192;
  signatureSlotSyncCommitteePeriodLessThan.in[1] <== signatureSlot;
  signatureSlotSyncCommitteePeriodLessThan.out === 1;

  component signatureSlotSyncCommitteePeriodGreaterThan = GreaterEqThan(64);
  signatureSlotSyncCommitteePeriodGreaterThan.in[0] <== signatureSlotSyncCommitteePeriod * 8192;
  signatureSlotSyncCommitteePeriodGreaterThan.in[1] <== signatureSlot - 8192;
  signatureSlotSyncCommitteePeriodGreaterThan.out === 1;

  component finalizedHeaderSlotSyncCommitteePeriodLessThan = LessEqThan(64);
  finalizedHeaderSlotSyncCommitteePeriodLessThan.in[0] <== finalizedHeaderSlotSyncCommitteePeriod * 8192;
  finalizedHeaderSlotSyncCommitteePeriodLessThan.in[1] <== prevHeaderFinalizedSlot;
  finalizedHeaderSlotSyncCommitteePeriodLessThan.out === 1;

  component finalizedHeaderSlotSyncCommitteePeriodGreaterThan = GreaterEqThan(64);
  finalizedHeaderSlotSyncCommitteePeriodGreaterThan.in[0] <== finalizedHeaderSlotSyncCommitteePeriod * 8192;
  finalizedHeaderSlotSyncCommitteePeriodGreaterThan.in[1] <== prevHeaderFinalizedSlot - 8192;
  finalizedHeaderSlotSyncCommitteePeriodGreaterThan.out === 1;

  component prevHeaderFinalizedSlotSSZ = SSZNum(64);
  prevHeaderFinalizedSlotSSZ.in <== prevHeaderFinalizedSlot;

  component nextHeaderSlotSSZ = SSZNum(64);
  nextHeaderSlotSSZ.in <== nextHeaderSlot;

  component isValidMerkleBranchPrevHeaderSlot = IsValidMerkleBranch(9);

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchPrevHeaderSlot.leaf[i] <== prevHeaderFinalizedSlotSSZ.out[i];
    isValidMerkleBranchPrevHeaderSlot.root[i] <== prevHeaderStateRoot[i];
  }

  for(var i = 0; i < 9; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchPrevHeaderSlot.branch[i][j] <== prevHeaderFinalizedSlotBranch[i][j];
    }
  }

  isValidMerkleBranchPrevHeaderSlot.index <== 840;
  isValidMerkleBranchPrevHeaderSlot.out === 1;


  component isValidMerkleBranchNextHeaderSlot = IsValidMerkleBranch(3);

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchNextHeaderSlot.leaf[i] <== nextHeaderSlotSSZ.out[i];
    isValidMerkleBranchNextHeaderSlot.root[i] <== nextHeaderHash[i];
  }

  for(var i = 0; i < 3; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchNextHeaderSlot.branch[i][j] <== nextHeaderSlotBranch[i][j];
    }
  }

  isValidMerkleBranchNextHeaderSlot.index <== 8;
  isValidMerkleBranchNextHeaderSlot.out === 1;


  component bitmaskContainsOnlyBools = BitmaskContainsOnlyBools(N);

  for(var i = 0; i < N; i++) {
    bitmaskContainsOnlyBools.bitmask[i] <== bitmask[i];
  }
  // Check if a supermajority of the ?validators signed the ???
  // aka check if there are 2/3 or more 1s in the bitmask
  component isSuperMajority = IsSuperMajority(N);

  for(var i = 0; i < N; i++) {
    isSuperMajority.bitmask[i] <== bitmask[i];
  }
  //Check if it returns 1
  isSuperMajority.out === 1;
  component computeDomain = ComputeDomain();

  for(var i = 0; i < 32; i++) {
    computeDomain.fork_version[i] <== fork_version[i];
  }

  for (var i = 0; i < 256; i++) {
    computeDomain.GENESIS_VALIDATORS_ROOT[i] <== GENESIS_VALIDATORS_ROOT[i];
  }

  for (var i = 0; i < 32; i++) {
    computeDomain.DOMAIN_SYNC_COMMITTEE[i] <== DOMAIN_SYNC_COMMITTEE[i];
  }

  component computeSigningRoot = ComputeSigningRoot();

  for(var i = 0; i < 256; i++) {
    computeSigningRoot.headerHash[i] <== nextHeaderHash[i];
  }
  //out of computeDomain -> input of computeSigningRoot
  for(var i = 0; i < 256; i++) {
    computeSigningRoot.domain[i] <== computeDomain.domain[i];
  }

  component hashToField = HashToField();

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

  component isValidMerkleBranchFinality = IsValidMerkleBranch(9);

  for(var i = 0; i < 9; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchFinality.branch[i][j] <== finalizedHeaderBranch[i][j];
    }
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchFinality.leaf[i] <== finalizedHeaderRoot[i];
    isValidMerkleBranchFinality.root[i] <== nextHeaderHash[i];
  }

  isValidMerkleBranchFinality.index <== 745;

  isValidMerkleBranchFinality.out === 1;

  component isValidMerkleBranchExecution = IsValidMerkleBranch(11);

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchExecution.leaf[i] <== execution_state_root[i];
    isValidMerkleBranchExecution.root[i] <== finalizedHeaderRoot[i];
  }

  for(var i = 0; i < 11; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchExecution.branch[i][j] <== execution_state_root_branch[i][j];
    }
  }

  isValidMerkleBranchExecution.index <== 3218;

  isValidMerkleBranchExecution.out === 1;

  component isValidMerkleBranchSyncCommittee = IsValidMerkleBranch(5);

  for(var i = 0; i < 5; i++) {
    for(var j = 0; j < 256; j++) {
      isValidMerkleBranchSyncCommittee.branch[i][j] <== syncCommitteeBranch[i][j];
    }
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchSyncCommittee.leaf[i] <== hasher.out[i];
  }

  for(var i = 0; i < 256; i++) {
    isValidMerkleBranchSyncCommittee.root[i] <== prevHeaderStateRoot[i];
  }

  component arePeriodsEqual = IsEqual();
  arePeriodsEqual.in[0] <== signatureSlotSyncCommitteePeriod;
  arePeriodsEqual.in[1] <== finalizedHeaderSlotSyncCommitteePeriod;

  isValidMerkleBranchSyncCommittee.index <== 55 - arePeriodsEqual.out;

  isValidMerkleBranchSyncCommittee.out === 1;

  component aggregateKeys = AggregateKeysBitmask(N);

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

  // bls.Verify
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

  component commitment = Sha256(1024);

  for(var i = 0; i < 256; i++) {
    commitment.in[i] <== prevHeaderHash[i];
  }

  for(var i = 0; i < 256; i++) {
    commitment.in[256 + i] <== nextHeaderHash[i];
  }

  for(var i = 0; i < 256; i++) {
    commitment.in[512 + i] <== finalizedHeaderRoot[i];
  }

  for(var i = 0; i < 256; i++) {
    commitment.in[768 + i] <== execution_state_root[i];
  }

  component bits2num1 = Bits2Num(253);

  for(var i = 0; i < 253; i++) {
    bits2num1.in[i] <== commitment.out[252 - i];
  }

  component bits2Num2 = Bits2Num(3);

  for(var i = 0; i < 3; i++) {
    bits2Num2.in[i] <== commitment.out[255 - i];
  }

  output_commitment[0] <== bits2num1.out;
  output_commitment[1] <== bits2Num2.out;
}
