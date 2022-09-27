import * as fs from 'fs';
import * as path from 'path';

import { groth16 } from 'snarkjs';
import { PointG1 } from '@noble/bls12-381';
import { BitArray } from '@chainsafe/ssz';
import { ssz } from '@chainsafe/lodestar-types';

import {
  formatPubkeysToPoints,
  formatBitmask,
  JSONHeader,
  FormatedJsonUpdate,
} from './format';

import { bigint_to_array, bytesToHex, hexToBytes } from './bls';

import * as constants from './constants';

import { exec } from 'child_process';
import { promisify } from 'util';

const promiseExec = promisify(exec);

interface JSONUpdate {
  attested_header: JSONHeader;
  next_sync_committee: {
    pubkeys: string[];
    aggregate_pubkey: string;
  };
  next_sync_committee_branch: string[];
  finalized_header: JSONHeader;
  finality_branch: string[];
  sync_aggregate: {
    sync_committee_bits: string;
    sync_committee_signature: string;
  };
  signature_slot: string;
}

export interface Proof {
  a: string[];
  b: string[][];
  c: string[];
}

export function getFilesInDir(_path: string) {
  let files: Buffer[] = [];
  const content = fs.readdirSync(_path, {
    encoding: 'utf-8',
    withFileTypes: true,
  });
  for (let f of content) {
    if (f.isDirectory()) {
      files = [...files, ...getFilesInDir(path.join(_path, f.name))];
    } else {
      files.push(fs.readFileSync(path.join(_path, f.name)));
    }
  }
  return files;
}

export function getAggregatePubkey(
  update1: JSONUpdate,
  update2: JSONUpdate,
): string {
  // Extract active participants public keys as G1 points
  const points: PointG1[] = formatPubkeysToPoints(update1.next_sync_committee);
  const bitmask: BitArray = formatBitmask(
    update2.sync_aggregate.sync_committee_bits,
  );

  const aggregatePubkey = points
    .filter((_, i) => bitmask.get(i))
    .reduce((prev, curr) => prev.add(curr))
    .toHex(true);
  return aggregatePubkey;
}

export function getMessage(root: Uint8Array, fork_version: Uint8Array) {
  const fork_data = ssz.phase0.ForkData.defaultValue();
  fork_data.currentVersion = fork_version;
  fork_data.genesisValidatorsRoot = constants.GENESIS_VALIDATORS_ROOT;
  const fork_data_root = ssz.phase0.ForkData.hashTreeRoot(fork_data);

  const domain = new Uint8Array(32);
  for (let i = 0; i < 4; i++) domain[i] = constants.DOMAIN_SYNC_COMMITTEE[i];
  for (let i = 0; i < 28; i++) domain[i + 4] = fork_data_root[i];

  const signing_data = ssz.phase0.SigningData.defaultValue();
  signing_data.objectRoot = root;
  signing_data.domain = domain;

  return ssz.phase0.SigningData.hashTreeRoot(signing_data);
}

// export async function getInputSignature(pubkey: string, signature: string, blockRoot: string) {
//     const pubkeyPoint = PointG1.fromHex(formatHex(pubkey));
//     pubkeyPoint.assertValidity();

//     const signaturePoint = PointG2.fromSignature(formatHex(signature));
//     signaturePoint.assertValidity();

//     const signing_root: Uint8Array = getMessage(hexToBytes(blockRoot), constants.GENESIS_FORK_VERSION);

//     const u: bigint[][] = await utils.hashToField(signing_root, 2);

//     const pubkeyArray: string[][] = [
//         bigint_to_array(55, 7, BigInt("0x" + pubkeyPoint.toAffine()[0].value.toString(16))),
//         bigint_to_array(55, 7, BigInt("0x" + pubkeyPoint.toAffine()[1].value.toString(16))),
//     ];

//     const signatureArray: string[][][] = [
//         [
//             bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[0].c0.value.toString(16))),
//             bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[0].c1.value.toString(16))),
//         ],
//         [
//             bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[1].c0.value.toString(16))),
//             bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[1].c1.value.toString(16))),
//         ],
//     ];

//     const hashArray: string[][][] = [
//         [
//             bigint_to_array(55, 7, BigInt("0x" + u[0][0].toString(16))),
//             bigint_to_array(55, 7, BigInt("0x" + u[0][1].toString(16))),
//         ],
//         [
//             bigint_to_array(55, 7, BigInt("0x" + u[1][0].toString(16))),
//             bigint_to_array(55, 7, BigInt("0x" + u[1][1].toString(16))),
//         ],
//     ];

//     return {
//         pubkey: pubkeyArray,
//         signature: signatureArray,
//         hash: hashArray
//     };
// }

export async function getSolidityProof(
  prevUpdate: FormatedJsonUpdate,
  update: FormatedJsonUpdate,
  network: string,
  generateProof?: boolean,
): Promise<Proof> {
  const period = compute_sync_committee_period(
    parseInt(update.attested_header.slot),
  );
  const proofsDir = `../../vendor/eth2-light-client-updates/${network}/proofs`;

  if (generateProof) {
    const proofsManifest = `${proofsDir}/manifest.json`;

    if (JSON.parse(fs.readFileSync(proofsManifest)).version < 2) {
      throw new Error('Please update vendor/eth2-light-client-updates');
    }

    fs.writeFileSync(
      'input.json',
      JSON.stringify(await getProofInput(prevUpdate, update)),
    );
    console.log('Witness generation...');
    console.log(
      await promiseExec(
        `../circom/build/light_client/light_client_cpp/light_client input.json witness.wtns`,
      ),
    );

    console.log('Proof generation...');
    console.log(
      (
        await promiseExec(
          `../../vendor/rapidsnark/build/prover ../circom/build/light_client/light_client_0.zkey witness.wtns ${proofsDir}/proof${period}.json ${proofsDir}/public${period}.json`,
        )
      ).stdout,
    );

    await promiseExec('rm input.json witness.wtns');
  }

  const proof = JSON.parse(
    fs.readFileSync(`${proofsDir}/proof${period}.json`).toString(),
  );
  const publicSignals = JSON.parse(
    fs.readFileSync(`${proofsDir}/public${period}.json`).toString(),
  );
  const calldata = await groth16.exportSolidityCallData(proof, publicSignals);

  const argv: string[] = calldata
    .replace(/["[\]\s]/g, '')
    .split(',')
    .map(x => BigInt(x).toString());

  const a = [argv[0], argv[1]];
  const b = [
    [argv[2], argv[3]],
    [argv[4], argv[5]],
  ];
  const c = [argv[6], argv[7]];

  return { a, b, c };
}

async function getProofInput(
  prevUpdate: FormatedJsonUpdate,
  update: FormatedJsonUpdate,
) {
  let points: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x =>
    PointG1.fromHex(x.slice(2)),
  );
  let bitmask = update.sync_aggregate.sync_committee_bits;
  const BeaconBlockHeader = ssz.phase0.BeaconBlockHeader;
  let block_header = BeaconBlockHeader.defaultValue();
  block_header.slot = Number.parseInt(update.attested_header.slot);
  block_header.proposerIndex = Number.parseInt(
    update.attested_header.proposer_index,
  );
  block_header.parentRoot = hexToBytes(update.attested_header.parent_root);
  block_header.stateRoot = hexToBytes(update.attested_header.state_root);
  block_header.bodyRoot = hexToBytes(update.attested_header.body_root);
  let hash = BeaconBlockHeader.hashTreeRoot(block_header);

  let prevBlock_header = BeaconBlockHeader.defaultValue();
  prevBlock_header.slot = Number.parseInt(prevUpdate.attested_header.slot);
  prevBlock_header.proposerIndex = Number.parseInt(
    prevUpdate.attested_header.proposer_index,
  );
  prevBlock_header.parentRoot = hexToBytes(
    prevUpdate.attested_header.parent_root,
  );
  prevBlock_header.stateRoot = hexToBytes(
    prevUpdate.attested_header.state_root,
  );
  prevBlock_header.bodyRoot = hexToBytes(prevUpdate.attested_header.body_root);
  let prevHash = BeaconBlockHeader.hashTreeRoot(prevBlock_header);

  let branch = prevUpdate.next_sync_committee_branch;
  let branchInput = branch.map(x =>
    BigInt(x).toString(2).padStart(256, '0').split(''),
  );

  let dataView = new DataView(new ArrayBuffer(8));
  dataView.setBigUint64(0, BigInt(prevBlock_header.slot));
  let slot = dataView.getBigUint64(0, true);
  slot = BigInt('0x' + slot.toString(16).padStart(16, '0').padEnd(64, '0'));

  dataView.setBigUint64(0, BigInt(prevBlock_header.proposerIndex));
  let proposer_index = dataView.getBigUint64(0, true);
  proposer_index = BigInt(
    '0x' + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'),
  );

  let nextBlockHeaderHash1 = BigInt('0x' + bytesToHex(hash))
    .toString(2)
    .padStart(256, '0')
    .slice(0, 253);
  let nextBlockHeaderHash2 = BigInt('0x' + bytesToHex(hash))
    .toString(2)
    .padStart(256, '0')
    .slice(253, 256);

  let prevBlockHeaderHash1 = BigInt('0x' + bytesToHex(prevHash))
    .toString(2)
    .padStart(256, '0')
    .slice(0, 253);
  let prevBlockHeaderHash2 = BigInt('0x' + bytesToHex(prevHash))
    .toString(2)
    .padStart(256, '0')
    .slice(253, 256);

  let input = {
    points: points.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
    prevHeaderHashNum: [
      BigInt('0b' + prevBlockHeaderHash1).toString(10),
      BigInt('0b' + prevBlockHeaderHash2).toString(10),
    ],
    nextHeaderHashNum: [
      BigInt('0b' + nextBlockHeaderHash1).toString(10),
      BigInt('0b' + nextBlockHeaderHash2).toString(10),
    ],
    slot: slot.toString(2).padStart(256, '0').split(''),
    proposer_index: proposer_index.toString(2).padStart(256, '0').split(''),
    parent_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.parentRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    state_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.stateRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    body_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.bodyRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    fork_version: BigInt('0x' + bytesToHex(constants.ALTAIR_FORK_VERSION))
      .toString(2)
      .padStart(32, '0')
      .split(''),
    aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey)
      .toString(2)
      .split(''),
    bitmask: bitmask,
    branch: branchInput,
    signature: update.sync_aggregate.sync_committee_signature,
  };

  return input;
}

function compute_sync_committee_period(slot: number) {
  return Math.floor(
    Math.floor(slot / constants.SLOTS_PER_EPOCH) /
      constants.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
  );
}
