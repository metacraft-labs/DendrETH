import * as fs from "fs";
import * as path from "path";

import { readFileSync, writeFileSync } from "fs";
import { exec } from "child_process";

import { groth16 } from "snarkjs";
import { PointG1, PointG2 } from "@noble/bls12-381";
import { BitArray, BitVectorType } from "@chainsafe/ssz";
import { ssz } from "@chainsafe/lodestar-types";

import { formatJSONBlockHeader, formatPubkeysToPoints, formatBitmask, JSONHeader, FormatedJsonUpdate } from "./format";

import {
  bigint_to_array,
  formatHex,
  hexToBytes,
  utils
} from "./bls";

import * as constants from "./constants";

import { exec } from "child_process";
import { promisify } from "util";

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

interface Proof {
  a: string[2];
  b: string[2][2];
  c: string[2];
}

export function getFilesInDir(_path: string) {
  let files: Buffer[] = [];
  const content = fs.readdirSync(_path, { encoding: 'utf-8', withFileTypes: true });
  for (let f of content) {
    if (f.isDirectory()) {
      files = [...files, ...getFilesInDir(path.join(_path, f.name))];
    } else {
      files.push(fs.readFileSync(path.join(_path, f.name)));
    }
  }
  return files;
}

export function getAggregatePubkey(update1: JSONUpdate, update2: JSONUpdate): string {
  // Extract active participants public keys as G1 points
  const points: PointG1[] = formatPubkeysToPoints(update1.next_sync_committee);
  const bitmask: BitArray = formatBitmask(update2.sync_aggregate.sync_committee_bits);

  const aggregatePubkey = points.filter((_, i) => bitmask.get(i)).reduce((prev, curr) => prev.add(curr)).toHex(true);
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

export async function getSolidityProof(prevUpdate: FormatedJsonUpdate, update: FormatedJsonUpdate, network: string, generateProof?: boolean): Proof {
  fs.writeFileSync("input.json", JSON.stringify(await getProofInput(prevUpdate, update)));

  const period = compute_sync_committee_period(parseInt(update.attested_header.slot));

  if (generateProof) {
    console.log("Witness generation...");
    console.log((await promiseExec("../circom/build/god_please/proof_efficient/proof_efficient_cpp/proof_efficient input.json witness.wtns")).stdout);

    console.log("Proof generation...");
    console.log((await promiseExec(`../../vendor/rapidsnark/build/prover ../circom/build/god_please/proof_efficient/proof_efficient_0.zkey witness.wtns data/${network}/proof${period}.json data/${NETWORK}/public${period}.json`)).stdout);

    await promiseExec("rm input.json witness.wtns");
  }

  const proof = JSON.parse(fs.readFileSync(`data/${network}/proof${period}.json`).toString());
  const publicSignals = JSON.parse(fs.readFileSync(`data/${network}/public${period}.json`).toString());
  const calldata = await groth16.exportSolidityCallData(proof, publicSignals);

  const argv = calldata.replace(/["[\]\s]/g, "").split(',').map(x => BigInt(x).toString());

  const a = [argv[0], argv[1]];
  const b = [[argv[2], argv[3]], [argv[4], argv[5]]];
  const c = [argv[6], argv[7]];

  return { a, b, c };
}

async function getProofInput(prevUpdate: FormatedJsonUpdate, update: FormatedJsonUpdate) {
  const pubkeyPoints: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x => PointG1.fromHex(formatHex(x)));

  const block_header = formatJSONBlockHeader(update.attested_header);
  const hash = ssz.phase0.BeaconBlockHeader.hashTreeRoot(block_header);

  const message = getMessage(hash, constants.ALTAIR_FORK_VERSION);
  const u = await utils.hashToField(message, 2);

  const input = {
    points: pubkeyPoints.map(x => [bigint_to_array(55, 7, x.toAffine()[0].value), bigint_to_array(55, 7, x.toAffine()[1].value)]),
    aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey).toString(2).split(''),
    bitmask: update.sync_aggregate.sync_committee_bits,
    signature: update.sync_aggregate.sync_committee_signature,
    hash: [
      [
        bigint_to_array(55, 7, u[0][0]),
        bigint_to_array(55, 7, u[0][1])
      ],
      [
        bigint_to_array(55, 7, u[1][0]),
        bigint_to_array(55, 7, u[1][1])
      ]
    ]
  };

  return input;
}

function compute_sync_committee_period(slot: number) {
  return parseInt(slot / constants.SLOTS_PER_EPOCH / constants.EPOCHS_PER_SYNC_COMMITTEE_PERIOD);
}
