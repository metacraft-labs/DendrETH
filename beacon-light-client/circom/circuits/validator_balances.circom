pragma circom 2.1.5;

include "bitify.circom";
include "hash_tree_root.circom";
include "hash_two.circom";
include "validator_hash_tree_root.circom";
include "is_valid_merkle_branch_out.circom";
include "is_valid_merkle_branch.circom";
include "ssz_num.circom";
include "./utils/arrays.circom";
include "./utils/bits.circom";
include "./utils/numerical.circom";

template CalculateBalancesSum(N) {
  signal input balances[(N \ 4) + 1][256];
  signal input balancesProofIndexesRemainders[N];
  signal input bitmask[N];

  signal output out;

  signal sums[N + 1];
  sums[0] <== 0;

  component bits2Num[N];
  component selector[64 * N];

  for (var i = 0; i < N; i++) {
    bits2Num[i] = Bits2Num(64);

    for(var j = 0; j < 64; j++) {
      selector[i * 64 + j] = Selector(256);

      for (var k = 0; k < 256; k++) {
        selector[i * 64 + j].in[k] <== balances[i \ 4][k];
      }

      selector[i * 64 + j].index <== 64 * balancesProofIndexesRemainders[i] + j;

      bits2Num[i].in[j] <== selector[i * 64 + j].out;
    }

    sums[i + 1] <== sums[i] + bitmask[i] * bits2Num[i].out;
  }

  out <== sums[N];
}

template ValidatorBalances(N) {
  signal input validatorsAccumulator[256];

  signal input validatorsPubkeys[N][384];
  signal input withdrawCredentials[N][256];
  signal input effectiveBalance[N][256];
  signal input slashed[N][256];
  signal input activationEligibilityEpoch[N][256];

  // Needed to check it is active validator
  signal input activationEpoch[N];
  signal input exitEpoch[N];
  signal input withdrawableEpoch[N][256];

  signal input validatorBranch[N][41][256];

  signal input validatorsIndexes[N];
  signal input validatorEth1DepositIndex[N];

  signal input balancesProofIndexes[(N \ 4) + 1];
  signal input balancesProofIndexesRemainders[N];

  signal input stateRoot[256];

  signal input slot;
  signal input slotBranch[5][256];

  signal input validatorsRoot[256];
  signal input validatorsBranch[5][256];

  signal input currentEth1DepositIndex;
  signal input eth1DepositIndexBranch[5][256];

  signal input balanceBranch[5][256];
  signal input balanceRoot[256];
  signal input balances[(N \ 4) + 1][256];
  signal input balancesBranches[(N \ 4) + 1][39][256];

  signal output commitment;

  signal currentEpoch <-- slot \ 32;
  // signal currentEpochRemainder <-- slot % 32;
  // slot === currentEpoch * 32 + currentEpochRemainder;

  signal epochHighestSlot <== currentEpoch * 32;

  // Should be LessThanOrEqualBitsCheck(64)([slot, epochHighestSlot + 32])
  signal slotLessThan <== LessThanBitsCheck(64)([slot, epochHighestSlot]);

  signal slotBits[256] <== SSZNum(64)(slot);

  IsValidMerkleBranch(5)(slotBranch, slotBits, stateRoot, 34);

  signal currentEth1DepositIndexBits[256] <== SSZNum(64)(currentEth1DepositIndex);
  IsValidMerkleBranch(5)(eth1DepositIndexBranch, currentEth1DepositIndexBits, stateRoot, 42);
  IsValidMerkleBranch(5)(validatorsBranch, validatorsRoot, stateRoot, 43);
  IsValidMerkleBranch(5)(balanceBranch, balanceRoot, stateRoot, 44);

  signal bitmask[N];

  // Signals are immutable and can be assigned only once
  signal pubkeyBits[N][512];
  signal activationEpochLte[N];
  signal exitEpochLte[N];
  signal validatorsHash[N][256];
  signal lte[N];
  signal isValidMerkleBranch[N];

  signal med[N];

  // verify merkle proof for existing validators
  for(var i = 0; i < N; i++) {

    for(var j = 0; j < 384; j++) {
      pubkeyBits[i][j] <== validatorsPubkeys[i][j];
    }

    for(var j = 384; j < 512; j++) {
      pubkeyBits[i][j] <== 0;
    }

    activationEpochLte[i] <== LessThan(64)([activationEpoch[i], currentEpoch]);
    exitEpochLte[i] <== LessThan(64)([currentEpoch, exitEpoch[i]]);

    validatorsHash[i] <== ValidatorHashTreeRoot()(Sha256(512)(pubkeyBits[i]), withdrawCredentials[i], effectiveBalance[i], slashed[i],
                                                         activationEligibilityEpoch[i], activationEpoch[i], exitEpoch[i], withdrawableEpoch[i]);

    lte[i] <== LessThanOrEqualBitsCheck(64)([validatorEth1DepositIndex[i], currentEth1DepositIndex]);

    isValidMerkleBranch[i] <== IsValidMerkleBranchOut(41)(validatorBranch[i], validatorsHash[i], validatorsRoot, 2199023255552 + validatorsIndexes[i]);

    med[i] <== lte[i] * activationEpochLte[i];

    bitmask[i] <== med[i] * exitEpochLte[i];

    isValidMerkleBranch[i] * lte[i] === lte[i];
  }

  for(var i = 0; i < N; i++) {
    DivisionVerification()(validatorsIndexes[i], 4, balancesProofIndexes[i \ 4], balancesProofIndexesRemainders[i]);
  }

  for(var i = 0; i < (N \ 4) + 1; i++) {
    IsValidMerkleBranch(39)(balancesBranches[i], balances[i], balanceRoot, 549755813888 + balancesProofIndexes[i]);
  }

  component validatorsHashTreeRoot = HashTreeRoot(N);

  signal leaveBits[N][512];
  signal validatorEth1DepositIndexBits[N][64];

  for(var i = 0; i < N; i++) {
    validatorEth1DepositIndexBits[i] <== Num2Bits(64)(validatorEth1DepositIndex[i]);

    for(var j = 0; j < 384; j++) {
      leaveBits[i][j] <== validatorsPubkeys[i][j];
    }

    for(var j = 384; j < 448; j++) {
      leaveBits[i][j] <== 0;
    }

    for(var j = 448; j < 512; j++) {
      leaveBits[i][j] <== validatorEth1DepositIndexBits[i][511 - j];
    }

    validatorsHashTreeRoot.leaves[i] <== Sha256(512)(leaveBits[i]);
  }

  validatorsHashTreeRoot.out === validatorsAccumulator;


  signal sum <== CalculateBalancesSum(N)(balances, balancesProofIndexesRemainders, bitmask);

  signal commitmentBits[768];

  signal sumBits[256] <== SSZNum(64)(sum);

  for(var i = 0; i < 256; i++) {
    commitmentBits[i] <== sumBits[i];
  }

  for(var i = 0; i < 256; i++) {
    commitmentBits[256 + i] <== validatorsAccumulator[i];
  }

  for(var i = 0; i < 256; i++) {
    commitmentBits[512 + i] <== stateRoot[i];
  }

  signal hashedCommitmentBits[256] <== Sha256(768)(commitmentBits);

  signal firstBits[253];

  for(var i = 0; i < 253; i++) {
    firstBits[i] <== hashedCommitmentBits[i];
  }

  commitment <== Bits2Num(253)(firstBits);
}
