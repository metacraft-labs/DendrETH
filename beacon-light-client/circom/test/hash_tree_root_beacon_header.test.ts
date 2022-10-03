import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { wasm } from './circuit_tester';
import { expect } from 'chai';
import * as update from '../../../vendor/eth2-light-client-updates/mainnet/updates/00294.json';
import { ssz } from '@chainsafe/lodestar-types';
import { formatJSONBlockHeader } from '../../solidity/test/utils/format';

describe('Hash tree root beacon header test', () => {
  it('Test1', async () => {
    const block_header = formatJSONBlockHeader(update.attested_header);

    let dataView = new DataView(new ArrayBuffer(8));
    dataView.setBigUint64(0, BigInt(block_header.slot));
    let slot = dataView.getBigUint64(0, true);
    slot = BigInt('0x' + slot.toString(16).padStart(16, '0').padEnd(64, '0'));

    dataView.setBigUint64(0, BigInt(block_header.proposerIndex));
    let proposer_index = dataView.getBigUint64(0, true);
    proposer_index = BigInt('0x' + proposer_index.toString(16).padEnd(64, '0'));

    let input = {
      slot: slot.toString(2).padStart(256, '0').split(''),
      proposer_index: proposer_index.toString(2).padStart(256, '0').split(''),
      parent_root: BigInt(
        '0x' + bytesToHex(block_header.parentRoot as Uint8Array),
      )
        .toString(2)
        .padStart(256, '0')
        .split(''),
      state_root: BigInt(
        '0x' + bytesToHex(block_header.stateRoot as Uint8Array),
      )
        .toString(2)
        .padStart(256, '0')
        .split(''),
      body_root: BigInt('0x' + bytesToHex(block_header.bodyRoot as Uint8Array))
        .toString(2)
        .padStart(256, '0')
        .split(''),
    };

    const hash = ssz.phase0.BeaconBlockHeader.hashTreeRoot(block_header);

    const circuit = await wasm(
      './scripts/hash_tree_root_beacon_header/hash_tree_root_beacon_header.circom',
    );

    const witness = await circuit.calculateWitness(input);
    let a = '';
    for (let i = 1; i <= 256; i++) {
      a += witness[i];
    }

    expect(
      BigInt('0x' + bytesToHex(hash))
        .toString(2)
        .padStart(256, '0'),
    ).to.be.eq(a);
  });
});
