import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { readFileSync, writeFileSync } from 'fs';
import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';

let validatorsJSON = JSON.parse(readFileSync('../../validators.json', 'utf-8'));

let validators = ssz.phase0.Validators.fromJson(
  validatorsJSON.data.map(x => x.validator),
);
let validatorsView = ssz.phase0.Validators.toViewDU(validators);
const validatorsTree = new Tree(validatorsView.node);
// pubkey 8n
// withdrawalCredentials: 9n
// effectiveBalance: 10n
// slashed 11n
// activationEligibilityEpoch 12n
// activationEpoch 13n
// exitEpoch 14n
// withdrawableEpoch 15n
const withdrawCredentials: string[][] = [];
const effectiveBalance: string[][] = [];
const slashed: string[][] = [];
const activationEligibilityEpoch: string[][] = [];
const activationEpoch: string[][] = [];
const exitEpoch: string[][] = [];
const withdrawableEpoch: string[][] = [];

for (let i = 0; i < 4; i++) {
  const validatorTree = new Tree(
    validatorsTree.getNode(ssz.phase0.Validators.getPathInfo([i]).gindex),
  );

  withdrawCredentials.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(9n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  effectiveBalance.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(10n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );
  slashed.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(11n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  activationEligibilityEpoch.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(12n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  activationEpoch.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(13n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  exitEpoch.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(14n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );

  withdrawableEpoch.push(
    BigInt('0x' + bytesToHex(validatorTree.getNode(15n).root))
      .toString(2)
      .padStart(256, '0')
      .split(''),
  );
}

const pubkeys = validators.slice(0, 4).map(x =>
  BigInt('0x' + bytesToHex(x.pubkey))
    .toString(2)
    .padStart(384, '0')
    .split(''),
);

writeFileSync(
  'scripts/validators_hash_tree_root/input.json',
  JSON.stringify({
    pubkeys,
    withdrawCredentials,
    effectiveBalance,
    slashed,
    activationEligibilityEpoch,
    activationEpoch,
    exitEpoch,
    withdrawableEpoch,
  }),
);
