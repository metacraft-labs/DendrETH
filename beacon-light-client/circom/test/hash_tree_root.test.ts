import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { wasm } from './circuit_tester';
import { expect } from 'chai';
import * as update from '../../../vendor/eth2-light-client-updates/mainnet/updates/00294.json';
import { ssz } from '@chainsafe/lodestar-types';
import { formatJSONBlockHeader } from '../../solidity/test/utils/format';
import { randomBytes } from 'crypto';
import { sha256 } from 'ethers/lib/utils';
import { formatHex } from '../../solidity/test/utils/bls';

describe('Hash tree root test', () => {
  it('Test1', async () => {
    const leaves = [...Array(4).keys()].map(x => bytesToHex(randomBytes(32)));

    const circuit = await wasm(
      './scripts/hash_tree_root/hash_tree_root.circom',
    );

    const input = {
      leaves: leaves.map(x =>
        BigInt('0x' + x)
          .toString(2)
          .padStart(256, '0')
          .split(''),
      ),
    };

    const witness = await circuit.calculateWitness(input);
    let a = '';
    for (let i = 1; i <= 256; i++) {
      a += witness[i];
    }

    let hash1 = sha256('0x' + leaves[0] + leaves[1]);
    let hash2 = sha256('0x' + leaves[2] + leaves[3]);

    let root = sha256(hash1 + formatHex(hash2));

    expect(BigInt(root).toString(2).padStart(256, '0')).to.be.eq(a);
  });
});
