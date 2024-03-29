import { writeFileSync } from 'fs';

import { sha256 } from 'ethers/lib/utils';
import { Tree } from '@chainsafe/persistent-merkle-tree';

import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { hashTreeRoot } from '@dendreth/utils/ts-utils/ssz-utils';

function bytesToBinaryArray(byteArray: Uint8Array): string[] {
  return BigInt(`0x${bytesToHex(byteArray)}`)
    .toString(2)
    .padStart(256, '0')
    .split('');
}

function getValidatorTree(validator, ssz): Tree {
  const validatorView = ssz.phase0.Validator.toViewDU(validator);
  return new Tree(validatorView.node);
}

// TODO: fix so it matches the test in beacon-light-client/solidity
export function calculateValidatorsAccumulator(
  validatorsPubkeys: Uint8Array[],
  eth1DepositIndexes: BigInt[],
) {
  const leaves: string[] = [];

  for (let i = 0; i < validatorsPubkeys.length; i++) {
    const validatorPubkey = bytesToHex(validatorsPubkeys[i])
      .padStart(96, '0')
      .padEnd(112, '0');
    const eth1DepositIndex =
      '0x' + eth1DepositIndexes[i].toString(16).padStart(16, '0');

    leaves.push(
      sha256('0x' + formatHex(validatorPubkey) + formatHex(eth1DepositIndex)),
    );
  }

  const validatorsAccumulator = hexToBytes(hashTreeRoot(leaves, leaves.length));

  return bytesToBinaryArray(validatorsAccumulator);
}

(async () => {
  const { ssz } = await import('@lodestar/types');

  const SIZE = 2;

  const beaconStateSZZ = await fetch(
    `http://testing.mainnet.beacon-api.nimbus.team/eth/v2/debug/beacon/states/6616005`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);
  const validators = beaconState.validators.slice(0, SIZE);

  const validatorsView = ssz.phase0.Validators.toViewDU(beaconState.validators);
  const validatorsTree = new Tree(validatorsView.node);

  const validatorZeroIndex = ssz.phase0.Validators.getPathInfo([0]).gindex;

  const validatorBranch = validators.map((_, i) =>
    validatorsTree
      .getSingleProof(validatorZeroIndex + BigInt(i))
      .map(bytesToBinaryArray),
  );

  const validatorsPubkeys = validators.map(validator =>
    bytesToBinaryArray(validator.pubkey),
  );

  const withdrawCredentials = validators
    .map(x => getValidatorTree(x, ssz).getNode(9n).root)
    .map(bytesToBinaryArray);

  const effectiveBalance = validators
    .map(x => getValidatorTree(x, ssz).getNode(10n).root)
    .map(bytesToBinaryArray);
  const slashed = validators
    .map(x => getValidatorTree(x, ssz).getNode(11n).root)
    .map(bytesToBinaryArray);

  const activationEligibilityEpoch = validators
    .map(x => getValidatorTree(x, ssz).getNode(12n).root)
    .map(bytesToBinaryArray);

  const activationEpoch = validators.map(validator =>
    validator.activationEpoch.toString(),
  );
  const exitEpoch = validators.map(validator =>
    validator.exitEpoch.toString() === 'Infinity'
      ? '18446744073709551615'
      : validator.exitEpoch.toString(),
  );

  const withdrawableEpoch = validators
    .map(x => getValidatorTree(x, ssz).getNode(15n).root)
    .map(bytesToBinaryArray);

  const validatorsIndexes = Array.from({ length: SIZE }, (_, i) => i);

  const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

  const slotBranch = beaconStateTree
    .getSingleProof(34n)
    .map(bytesToBinaryArray);

  const validatorsBranch = beaconStateTree
    .getSingleProof(43n)
    .map(bytesToBinaryArray);

  const eth1DepositIndexBranch = beaconStateTree
    .getSingleProof(42n)
    .map(bytesToBinaryArray);

  const balanceBranch = beaconStateTree
    .getSingleProof(44n)
    .map(bytesToBinaryArray);

  const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const balancesTree = new Tree(balancesView.node);

  const balanceZeroIndex = ssz.capella.BeaconState.fields.balances.getPathInfo([
    0,
  ]).gindex;

  const balances: string[][] = [];
  const balancesBranches: string[][][] = [];

  for (let i = 0n; i <= Math.floor(SIZE / 4); i++) {
    balances.push(
      bytesToBinaryArray(balancesTree.getNode(balanceZeroIndex + i).root),
    );

    balancesBranches.push(
      balancesTree.getSingleProof(balanceZeroIndex + i).map(bytesToBinaryArray),
    );
  }

  const balancesProofIndexes = validatorsIndexes
    .filter((_, i) => i % 4 === 0)
    .map(val => Math.floor(val / 4).toString());

  const balancesProofIndexesRemainders = validatorsIndexes.map(
    validatorIndex => validatorIndex % 4,
  );

  // Fake values
  const validatorEth1DepositIndex = Array.from({ length: SIZE }, (_, i) =>
    BigInt(i).toString(),
  );

  const inputJSON = {
    validatorsAccumulator: calculateValidatorsAccumulator(
      validators.map(x => x.pubkey),
      validatorEth1DepositIndex.map(x => BigInt(x)),
    ),
    validatorsPubkeys,
    withdrawCredentials,
    effectiveBalance,
    slashed,
    activationEligibilityEpoch,
    activationEpoch,
    exitEpoch,
    withdrawableEpoch,
    validatorBranch,
    validatorsIndexes,
    validatorEth1DepositIndex,
    balancesProofIndexes,
    balancesProofIndexesRemainders,
    stateRoot: bytesToBinaryArray(
      ssz.capella.BeaconState.hashTreeRoot(beaconState),
    ),
    slot: beaconState.slot,
    slotBranch,
    validatorsRoot: bytesToBinaryArray(
      ssz.phase0.Validators.hashTreeRoot(beaconState.validators),
    ),
    validatorsBranch,
    currentEth1DepositIndex: beaconState.eth1DepositIndex,
    eth1DepositIndexBranch,
    balanceBranch,
    balanceRoot: bytesToBinaryArray(
      ssz.capella.BeaconState.fields.balances.hashTreeRoot(
        beaconState.balances,
      ),
    ),
    balances,
    balancesBranches,
  };

  writeFileSync('input.json', JSON.stringify(inputJSON, null, 2));
})();
