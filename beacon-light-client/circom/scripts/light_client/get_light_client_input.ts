import { PointG1, PointG2 } from "@noble/bls12-381";
import { bigint_to_array, bytesToHex, formatHex, hexToBytes, utils } from "../../../../libs/typescript/ts-utils/bls";
import { ssz } from "@chainsafe/lodestar-types";
import { writeFileSync } from "fs";
import { BitVectorType } from "@chainsafe/ssz";
import * as path from "path";
import { getFilesInDir } from "../../../../libs/typescript/ts-utils/data";
import { SyncCommittee } from "@chainsafe/lodestar-types/lib/altair/sszTypes";
import * as constants from "../../../solidity/test/utils/constants";

async function getProof(prevUpdate, update) {
  let points: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x => PointG1.fromHex(x.slice(2)));
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(update.sync_aggregate.sync_committee_bits);
  let signature: PointG2 = PointG2.fromSignature(formatHex(update.sync_aggregate.sync_committee_signature));
  const BeaconBlockHeader = ssz.phase0.BeaconBlockHeader;
  let block_header = BeaconBlockHeader.defaultValue();
  block_header.slot = Number.parseInt(update.attested_header.slot);
  block_header.proposerIndex = Number.parseInt(update.attested_header.proposer_index);
  block_header.parentRoot = hexToBytes(update.attested_header.parent_root);
  block_header.stateRoot = hexToBytes(update.attested_header.state_root);
  block_header.bodyRoot = hexToBytes(update.attested_header.body_root);
  let hash = BeaconBlockHeader.hashTreeRoot(block_header);

  let prevBlock_header = BeaconBlockHeader.defaultValue();
  prevBlock_header.slot = Number.parseInt(prevUpdate.attested_header.slot);
  prevBlock_header.proposerIndex = Number.parseInt(prevUpdate.attested_header.proposer_index);
  prevBlock_header.parentRoot = hexToBytes(prevUpdate.attested_header.parent_root);
  prevBlock_header.stateRoot = hexToBytes(prevUpdate.attested_header.state_root);
  prevBlock_header.bodyRoot = hexToBytes(prevUpdate.attested_header.body_root);
  let prevHash = BeaconBlockHeader.hashTreeRoot(prevBlock_header);

  console.log(prevHash);

  let branch = prevUpdate.next_sync_committee_branch;
  branch = branch.map(x => BigInt(x).toString(2).padStart(256, '0').split(''));

  let dataView = new DataView(new ArrayBuffer(8));
  dataView.setBigUint64(0, BigInt(prevBlock_header.slot));
  let slot = dataView.getBigUint64(0, true);
  slot = BigInt("0x" + slot.toString(16).padStart(16, '0').padEnd(64, '0'));

  dataView.setBigUint64(0, BigInt(prevBlock_header.proposerIndex));
  let proposer_index = dataView.getBigUint64(0, true);
  proposer_index = BigInt("0x" + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'));

  let nextBlockHeaderHash1 = BigInt("0x" + bytesToHex(hash)).toString(2).padStart(256, '0').slice(0, 253);
  let nextBlockHeaderHash2 = BigInt("0x" + bytesToHex(hash)).toString(2).padStart(256, '0').slice(253, 256);


  let prevBlockHeaderHash1 = BigInt("0x" + bytesToHex(prevHash)).toString(2).padStart(256, '0').slice(0, 253);
  let prevBlockHeaderHash2 = BigInt("0x" + bytesToHex(prevHash)).toString(2).padStart(256, '0').slice(253, 256);

  let input = {
    points: points.map(x => [bigint_to_array(55, 7, x.toAffine()[0].value), bigint_to_array(55, 7, x.toAffine()[1].value)]),
    prevHeaderHashNum: [BigInt('0b' + prevBlockHeaderHash1).toString(10), BigInt('0b' + prevBlockHeaderHash2).toString(10)],
    nextHeaderHashNum: [BigInt('0b' + nextBlockHeaderHash1).toString(10), BigInt('0b' + nextBlockHeaderHash2).toString(10)],
    slot: slot.toString(2).padStart(256, '0').split(''),
    proposer_index: proposer_index.toString(2).padStart(256, '0').split(''),
    parent_root: BigInt("0x" + bytesToHex(prevBlock_header.parentRoot as Uint8Array)).toString(2).padStart(256, '0').split(''),
    state_root: BigInt("0x" + bytesToHex(prevBlock_header.stateRoot as Uint8Array)).toString(2).padStart(256, '0').split(''),
    body_root: BigInt("0x" + bytesToHex(prevBlock_header.bodyRoot as Uint8Array)).toString(2).padStart(256, '0').split(''),
    fork_version: BigInt("0x" + bytesToHex(constants.ALTAIR_FORK_VERSION)).toString(2).padStart(32, '0').split(''),
    aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey).toString(2).split(''),
    bitmask: bitmask.toBoolArray().map(x => x ? '1' : '0'),
    branch: branch,
    signature: [
      [
        bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[0].c1.value)
      ],
      [
        bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[1].c1.value)
      ]
    ]
  };

  return input;
}

(async () => {
  const UPDATES = getFilesInDir(
    path.join(__dirname, "../../../../", "vendor", "eth2-light-client-updates", "mainnet", "updates")
  );

  let prevUpdate = UPDATES[4];

  for (let update of UPDATES.slice(5, 6)) {
    writeFileSync(path.join(__dirname, "input.json"), JSON.stringify(await getProof(JSON.parse(prevUpdate.toString()), JSON.parse(update as unknown as string))));
  }
})();

