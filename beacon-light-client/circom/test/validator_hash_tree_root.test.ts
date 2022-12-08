import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { wasm } from './circuit_tester';
import { expect } from 'chai';
import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { sha256 } from 'ethers/lib/utils';
import { Fp, PointG1 } from '@noble/bls12-381';
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
    let validator = validators[0];
    validator.exitEpoch = 16609;
    const validatorTree = new Tree(
      ssz.phase0.Validator.toViewDU(validator).node,
    );
    let withdrawalCredentialsHex = bytesToHex(validatorTree.getNode(9n).root);

    let effectifeBalanceHex = bytesToHex(validatorTree.getNode(10n).root);

    let withdrawableEpochHex = bytesToHex(validatorTree.getNode(15n).root);

    const circuit = await wasm(
      './scripts/validator_hash_tree_root/validator_hash_tree_root.circom',
    );

    console.log(bytesToHex(ssz.UintNum64.serialize(16609)));

    const input = {
      pubkey: BigInt('0x' + PointG1.fromHex(validator.pubkey).toHex(true))
        .toString(2)
        .padStart(384, '0')
        .split(''),
      withdrawCredentials: BigInt('0x' + withdrawalCredentialsHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      effectiveBalance: BigInt('0x' + effectifeBalanceHex)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      slashed: Number(validator.slashed),
      activationEligibilityEpoch: validator.activationEligibilityEpoch,
      activationEpoch: validator.activationEpoch,
      exitEpoch: validator.exitEpoch.toString() === 'Infinity' ? '18446744073709551615' : validator.exitEpoch.toString(),
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
