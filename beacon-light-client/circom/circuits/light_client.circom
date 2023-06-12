pragma circom 2.1.5;

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

  signal input prevFinalizedHeaderRoot[256];
  signal input prevFinalizedHeaderRootBranch[9][256];

  signal input prevHeaderFinalizedStateRoot[256];
  signal input prevHeaderFinalizedStateRootBranch[3][256];

  signal input prevHeaderFinalizedSlot;
  signal input prevHeaderFinalizedSlotBranch[3][256];

  signal input nextHeaderSlot;
  signal input nextHeaderSlotBranch[3][256];

  signal input signatureSlot;

  signal input signatureSlotSyncCommitteePeriod;
  signal input finalizedHeaderSlotSyncCommitteePeriod;

  signal input finalizedHeaderRoot[256];
  signal input finalizedHeaderBranch[9][256];

  signal input execution_state_root[256];
  signal input execution_state_root_branch[11][256];

  // Exposed as public via domain
  signal input fork_version[32];
  signal input GENESIS_VALIDATORS_ROOT[256];
  signal input DOMAIN_SYNC_COMMITTEE[32];

  signal input points[N][2][K];
  signal input aggregatedKey[384];
  signal input syncCommitteeBranch[5][256];

  signal input bitmask[N];
  signal input signature[2][2][K];

  signal output output_commitment[2];

  signal signatureSlotGreaterThanNext <== GreaterThan(64)([signatureSlot,nextHeaderSlot]);
  signatureSlotGreaterThanNext === 1;

  signal nextHeaderSlotGreaterThanPrevFinalized <== GreaterThan(64)([nextHeaderSlot,prevHeaderFinalizedSlot]);
  nextHeaderSlotGreaterThanPrevFinalized === 1;

  signal signatureSlotSyncCommitteePeriodLessThan <== LessEqThan(64)([signatureSlotSyncCommitteePeriod * 8192,signatureSlot]);
  signatureSlotSyncCommitteePeriodLessThan === 1;

  signal signatureSlotSyncCommitteePeriodGreaterThan <== GreaterEqThan(64)([signatureSlotSyncCommitteePeriod * 8192,signatureSlot - 8192]);
  signatureSlotSyncCommitteePeriodGreaterThan === 1;

  signal finalizedHeaderSlotSyncCommitteePeriodLessThan <== LessEqThan(64)([finalizedHeaderSlotSyncCommitteePeriod * 8192,prevHeaderFinalizedSlot]);
  finalizedHeaderSlotSyncCommitteePeriodLessThan === 1;

  signal finalizedHeaderSlotSyncCommitteePeriodGreaterThan <== GreaterEqThan(64)([finalizedHeaderSlotSyncCommitteePeriod * 8192,prevHeaderFinalizedSlot - 8192]);
  finalizedHeaderSlotSyncCommitteePeriodGreaterThan === 1;

  signal signaturePeriodNotMoreThanOnePeriodAboveFinalizedPeriod <== GreaterEqThan(64)([finalizedHeaderSlotSyncCommitteePeriod+1,signatureSlotSyncCommitteePeriod]);
  signaturePeriodNotMoreThanOnePeriodAboveFinalizedPeriod === 1;

  signal prevHeaderFinalizedSlotSSZ[256] <== SSZNum(64)(prevHeaderFinalizedSlot);

  signal nextHeaderSlotSSZ[256] <== SSZNum(64)(nextHeaderSlot);

  IsValidMerkleBranch(3)(prevHeaderFinalizedSlotBranch,prevHeaderFinalizedSlotSSZ,prevFinalizedHeaderRoot,8);

  IsValidMerkleBranch(3)(prevHeaderFinalizedStateRootBranch,prevHeaderFinalizedStateRoot,prevFinalizedHeaderRoot,11);

  IsValidMerkleBranch(3)(nextHeaderSlotBranch,nextHeaderSlotSSZ,nextHeaderHash,8);

  BitmaskContainsOnlyBools(N)(bitmask);

  // Check if there is a supermajority in the bitmask
  IsSuperMajority(N)(bitmask);

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

  IsValidMerkleBranch(9)(prevFinalizedHeaderRootBranch, prevFinalizedHeaderRoot, prevHeaderHash, 745);

  IsValidMerkleBranch(9)(finalizedHeaderBranch, finalizedHeaderRoot, nextHeaderHash, 745);

  IsValidMerkleBranch(11)(execution_state_root_branch, execution_state_root, finalizedHeaderRoot, 3218);

  signal arePeriodsEqual <== IsEqual()([signatureSlotSyncCommitteePeriod,finalizedHeaderSlotSyncCommitteePeriod]);

  IsValidMerkleBranch(5)(syncCommitteeBranch,hasher.out,prevHeaderFinalizedStateRoot,55-arePeriodsEqual);

  signal aggregateKeys[2][K] <== AggregateKeysBitmask(N,K)(points,bitmask);

  // bls.Verify
  CoreVerifyPubkeyG1(55, K)(aggregateKeys, signature, hashToField);

  component commitment = Sha256(1536);

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

  for(var i = 0; i < 192; i++) {
    commitment.in[1024 + i] <== 0;
  }

  signal nextHeaderSlotBits[64] <== Num2Bits(64)(nextHeaderSlot);

  for(var i = 192; i < 256; i++) {
    commitment.in[1024 + i] <== nextHeaderSlotBits[255 - i];
  }

  for(var i = 0; i < 256; i++) {
    commitment.in[1280 + i] <== computeDomain[i];
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
