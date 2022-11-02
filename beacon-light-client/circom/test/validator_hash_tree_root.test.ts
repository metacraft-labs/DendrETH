import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { wasm } from './circuit_tester';
import { expect } from 'chai';
import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { sha256 } from 'ethers/lib/utils';
import { formatHex } from '../../solidity/test/utils/bls';
import { readFileSync } from 'fs';

describe('Validator hash tree root test', () => {
  it('Test1', async () => {
    let validatorsJSON = JSON.parse(
      readFileSync('../../validators.json', 'utf-8'),
    );
    let validators = ssz.phase0.Validators.fromJson(
      validatorsJSON.data.map(x => x.validator),
    );
    const validatorTree = new Tree(
      ssz.phase0.Validator.toViewDU(validators[0]).node,
    );

    let pubkeyHex = bytesToHex(validatorTree.getNode(8n).root);
    let withdrawalCredentialsHex = bytesToHex(validatorTree.getNode(9n).root);

    let effectifeBalanceHex = bytesToHex(validatorTree.getNode(10n).root);
    let slashedHex = bytesToHex(validatorTree.getNode(11n).root);

    let activationEligibilityEpochHex = bytesToHex(
      validatorTree.getNode(12n).root,
    );
    let activationEpochHex = bytesToHex(validatorTree.getNode(13n).root);

    let exitEpochHex = bytesToHex(validatorTree.getNode(14n).root);
    let withdrawableEpochHex = bytesToHex(validatorTree.getNode(15n).root);

    const circuit = await wasm(
      './scripts/validator_hash_tree_root/validator_hash_tree_root.circom',
    );

    const input = {
      pubkey: BigInt('0x' + pubkeyHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      withdrawCredentials: BigInt('0x' + withdrawalCredentialsHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      effectiveBalance: BigInt('0x' + effectifeBalanceHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      slashed: BigInt('0x' + slashedHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      activationEligibilityEpoch: BigInt('0x' + activationEligibilityEpochHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      activationEpoch: BigInt('0x' + activationEpochHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      exitEpoch: BigInt('0x' + exitEpochHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      withdrawableEpoch: BigInt('0x' + withdrawableEpochHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
    };

    const witness = await circuit.calculateWitness(input);
    let a = '';
    for (let i = 1; i <= 256; i++) {
      a += witness[i];
    }

    const root = bytesToHex(validatorTree.root);

    expect(
      BigInt('0x' + root)
        .toString(2)
        .padStart(256, '0'),
    ).to.be.eq(a);
  });
});
