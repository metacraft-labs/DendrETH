import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { BitVectorType } from '@chainsafe/ssz';
import { sha256 } from 'ethers/lib/utils';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
const fs = require('fs');

import { buildPoseidon, buildPoseidonReference } from 'circomlibjs';

import { numberToBytesBE } from '@noble/bls12-381/math';
import { exit } from 'process';
(async () => {
  const { DenebClient } = await import('telepathyx/src/operatorx/deneb');
  const { ChainId } = await import('telepathyx/src/operatorx/config');

  const { ssz } = await import('@lodestar/types');

  const denebClient = new DenebClient(
    'http://gpu-server-001:5052',
    ChainId.Sepolia,
  );

  await getStepUpdate(denebClient, ssz, 5239744);
  await getRotateUpdate(denebClient, ssz, 5239744);

  // let rotate = await denebClient.getRotateUpdate(5239744);
  // let finalizedBlock = await denebClient.getBlock(5239744);

  // let pubkeysBytes = rotate.nextSyncCommittee.pubkeys;
  // let aggregatePubkeyBytesX = rotate.nextSyncCommittee.aggregatePubkey;
  // let pubkeysBigIntX = rotate.nextSyncCommittee.pubkeys
  //   .map(x => PointG1.fromHex(x))
  //   .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  // let pubkeysBigIntY = rotate.nextSyncCommittee.pubkeys
  //   .map(x => PointG1.fromHex(x))
  //   .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  // let syncCommitteeSSZ =
  //   ssz.deneb.BeaconState.fields.nextSyncCommittee.hashTreeRoot(
  //     rotate.nextSyncCommittee,
  //   );

  // let syncCommitteeBranch = rotate.nextSyncCommitteeBranch;

  // let syncCommitteePoseidon = getPoseidonInputs();

  // let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
  // let finalizedSlot = ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(
  //   finalizedBlock.slot,
  // );
  // let finalizedProposerIndex = finalizedBlock.proposerIndex;
  // let finalizedParentRoot = finalizedBlock.parentRoot;
  // let finalizedStateRoot = finalizedBlock.stateRoot;
  // let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
  //   finalizedBlock.body as any,
  // );
})();
async function getStepUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  const step = await denebClient.getStepUpdate(slot);
  let pubkeysX = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  let pubkeysY = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  const SyncCommitteeBits = new BitVectorType(512);

  let aggregationBits = step.syncAggregate.syncCommitteeBits
    .toBoolArray()
    .map(x => (x ? '1' : '0'));

  let signaturePoint = PointG2.fromSignature(
    bytesToHex(step.syncAggregate.syncCommitteeSignature),
  );
  let signature = [
    [
      bigint_to_array(55, 7, signaturePoint.toAffine()[0].c0.value),
      bigint_to_array(55, 7, signaturePoint.toAffine()[0].c1.value),
    ],
    [
      bigint_to_array(55, 7, signaturePoint.toAffine()[1].c0.value),
      bigint_to_array(55, 7, signaturePoint.toAffine()[1].c1.value),
    ],
  ];
  let sha256_fork_version = sha256(
    '0x' +
      bytesToHex(step.forkVersion) +
      bytesToHex(step.genesisValidatorsRoot),
  );

  const DOMAIN_SYNC_COMMITTEE = '07000000'; //removed the x0
  let domain = DOMAIN_SYNC_COMMITTEE + sha256_fork_version.slice(2, 58);
  let signing_root = sha256(
    '0x' + bytesToHex(step.attestedHeaderRoot) + domain,
  );
  let participation = aggregationBits
    .map(x => Number(x))
    .reduce((a, b) => a + b, 0);
  let participationBytes = numberToBytesBE(
    toLittleEndian(BigInt(participation)),
    32,
  );
  let syncCommitteePoseidon = await getPoseidonInputs(denebClient, slot);
  let syncCommitteePoseidonInHex = BigInt(syncCommitteePoseidon).toString(16);
  let syncCommitteePoseidonBytes = hexToBytes(syncCommitteePoseidonInHex);

  let finalityBranch = step.finalityBranch;
  let executionStateRoot = step.executionStateRoot;
  let executionStateBranch = step.executionStateBranch;
  let attestedHeaderRoot = step.attestedHeaderRoot;
  let attestedSlot = step.attestedBlock.slot;
  let attestedSlotBytes = numberToBytesBE(
    toLittleEndian(BigInt(attestedSlot)),
    32,
  );
  let attestedProposerIndex = step.attestedBlock.proposerIndex;
  let attestedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(attestedProposerIndex)),
    32,
  );
  let attestedParentRoot = step.attestedBlock.parentRoot;
  let attestedStateRoot = step.attestedBlock.stateRoot;
  let attestedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.attestedBlock.body as any,
  );
  let finalizedHeaderRoot = step.finalizedHeaderRoot;
  let finalizedSlot = step.finalizedBlock.slot;
  let finalizedSlotBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedSlot)),
    32,
  );
  let finalizedProposerIndex = step.finalizedBlock.proposerIndex;
  let finalizedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedProposerIndex)),
    32,
  );
  let finalizedParentRoot = step.finalizedBlock.parentRoot;
  let finalizedStateRoot = step.finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.finalizedBlock.body as any,
  );

  let sha0 = sha256(
    '0x' + bytesToHex(attestedSlotBytes) + bytesToHex(finalizedSlotBytes),
  );
  let sha1 = sha256(sha0 + bytesToHex(finalizedHeaderRoot));
  let sha2 = sha256(sha1 + bytesToHex(participationBytes));
  let sha3 = sha256(sha2 + bytesToHex(executionStateRoot));
  let sha4 = sha256(sha3 + syncCommitteePoseidonInHex);
  let publicInputsRoot = getFirst253Bits(hexToBytes(sha4));

  // Create the JSON object in the specified order
  const jsonOutput = {
    /* Attested Header */
    attestedHeaderRoot: Array.from(Buffer.from(attestedHeaderRoot)),
    attestedSlot: Array.from(Buffer.from(attestedSlotBytes)),
    attestedProposerIndex: Array.from(Buffer.from(attestedProposerIndexBytes)),
    attestedParentRoot: Array.from(Buffer.from(attestedParentRoot)),
    attestedStateRoot: Array.from(Buffer.from(attestedStateRoot)),
    attestedBodyRoot: Array.from(Buffer.from(attestedBodyRoot)),
    /* Finalized Header */
    finalizedHeaderRoot: Array.from(Buffer.from(finalizedHeaderRoot)),
    finalizedSlot: Array.from(Buffer.from(finalizedSlotBytes)),
    finalizedProposerIndex: Array.from(
      Buffer.from(finalizedProposerIndexBytes),
    ),
    finalizedParentRoot: Array.from(Buffer.from(finalizedParentRoot)),
    finalizedStateRoot: Array.from(Buffer.from(finalizedStateRoot)),
    finalizedBodyRoot: Array.from(Buffer.from(finalizedBodyRoot)),
    /* Sync Committee Protocol */
    pubkeysX,
    pubkeysY,
    aggregationBits,
    signature,
    domain: Array.from(Buffer.from(hexToBytes(domain))),
    signingRoot: Array.from(Buffer.from(hexToBytes(signing_root))),
    participation: getFirst253Bits(participationBytes),
    syncCommitteePoseidon: getFirst253Bits(syncCommitteePoseidonBytes),
    /* Finality Proof */
    finalityBranch: finalityBranch.map(finalityB =>
      Array.from(Buffer.from(finalityB)),
    ),
    /* Execution State Proof */
    executionStateRoot: Array.from(Buffer.from(executionStateRoot)),
    executionStateBranch: executionStateBranch.map(
      executionSB => Array.from(Buffer.from(executionSB)), //9 not 8
    ),
    /* Commitment to Public Inputs */
    publicInputsRoot: publicInputsRoot,
  };
  // Write the JSON output to a file
  const outputFilePath = 'InputForStepUpdate.json';
  await fs.writeFile(
    outputFilePath,
    JSON.stringify(jsonOutput, null, 2),
    'utf-8',
  );
  console.log(`JSON file has been written to ${outputFilePath}`);
}

async function getRotateUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  let rotate = await denebClient.getRotateUpdate(slot);
  let finalizedBlock = await denebClient.getBlock(slot);

  let pubkeysBytes = rotate.nextSyncCommittee.pubkeys;
  let aggregatePubkeyBytesX = rotate.nextSyncCommittee.aggregatePubkey;
  let pubkeysBigIntX = rotate.nextSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  let pubkeysBigIntY = rotate.nextSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  let syncCommitteeSSZ =
    ssz.deneb.BeaconState.fields.nextSyncCommittee.hashTreeRoot(
      rotate.nextSyncCommittee,
    );

  let syncCommitteeBranch = rotate.nextSyncCommitteeBranch;

  let syncCommitteePoseidon = await getPoseidonInputs(denebClient, slot);
  let syncCommitteePoseidonInHex = BigInt(syncCommitteePoseidon).toString(16);
  let syncCommitteePoseidonBytes = hexToBytes(syncCommitteePoseidonInHex);

  let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
  let finalizedSlot = ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(
    finalizedBlock.slot,
  );
  let finalizedProposerIndex = finalizedBlock.proposerIndex;
  let finalizedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedProposerIndex)),
    32,
  );
  let finalizedParentRoot = finalizedBlock.parentRoot;
  let finalizedStateRoot = finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    finalizedBlock.body as any,
  );
  const jsonOutput = {
    pubkeysBytes: pubkeysBytes.map(pubkeys => Array.from(Buffer.from(pubkeys))),
    aggregatePubkeyBytesX: Array.from(Buffer.from(aggregatePubkeyBytesX)),
    pubkeysBigIntX,
    pubkeysBigIntY,
    syncCommitteeSSZ: Array.from(Buffer.from(syncCommitteeSSZ)),
    syncCommitteeBranch: syncCommitteeBranch.map(branch =>
      Array.from(Buffer.from(branch)),
    ),
    syncCommitteePoseidon: getFirst253Bits(syncCommitteePoseidonBytes),
    finalizedHeaderRoot: Array.from(Buffer.from(finalizedHeaderRoot)),
    finalizedSlot: Array.from(Buffer.from(finalizedSlot)),
    finalizedProposerIndex: Array.from(
      Buffer.from(finalizedProposerIndexBytes),
    ),
    finalizedParentRoot: Array.from(Buffer.from(finalizedParentRoot)),
    finalizedStateRoot: Array.from(Buffer.from(finalizedStateRoot)),
    finalizedBodyRoot: Array.from(Buffer.from(finalizedBodyRoot)),
  };
  // Write the JSON output to a file
  const outputFilePath = 'InputForRotateUpdate.json';
  await fs.writeFile(
    outputFilePath,
    JSON.stringify(jsonOutput, null, 2),
    'utf-8',
  );

  console.log(`JSON file has been written to ${outputFilePath}`);
}

async function getPoseidonInputs(denebClient: DenebClient, slot: number) {
  const step = await denebClient.getStepUpdate(slot);
  let pubkeys = step.currentSyncCommittee.pubkeys.map(pKeys =>
    bytesToHex(Uint8Array.from(Buffer.from(pKeys))),
  );

  let pubKeyPoints = pubkeys.map(x => PointG1.fromHex(formatHex(x)).toAffine());
  let pubKeysArray = pubKeyPoints.map(x => [
    bigint_to_array(55, 7, x[0].value),
    bigint_to_array(55, 7, x[1].value),
  ]);

  let pubKeysArrayStr = pubKeysArray.map(pair => [
    pair[0].map(bigInt => bigInt.toString()),
    pair[1].map(bigInt => bigInt.toString()),
  ]);

  let jsonObject = {
    pubkeys: pubKeysArrayStr,
  };

  // // Convert the JSON object to a JSON string
  // let jsonString = JSON.stringify(jsonObject, null, 2); // Pretty print with 2-space indentation

  // // Write JSON string to a file
  // fs.writeFile('poseidonInputs.json', jsonString, 'utf8', err => {
  //   if (err) {
  //     console.error('Error writing file:', err);
  //   } else {
  //     console.log('File has been saved.');
  //   }
  // });
  let poseidon = await buildPoseidonReference();

  let poseidonValFlat: string[] = [];
  for (let i = 0; i < 512; i++) {
    for (let j = 0; j < 7; j++)
      for (let l = 0; l < 2; l++) {
        poseidonValFlat[i * 7 * 2 + j * 2 + l] = pubKeysArray[i][l][j];
      }
  }

  let prev: any = 0;

  const LENGTH = 512 * 2 * 7;
  const NUM_ROUNDS = LENGTH / 16;
  for (let i = 0; i < NUM_ROUNDS; i++) {
    let inputs: any[] = [];
    for (let j = 0; j < 16; j++) {
      inputs.push(poseidonValFlat[i * 16 + j]);
    }
    if (i < NUM_ROUNDS - 1) {
      prev = poseidon(inputs, prev, 1);
    } else {
      prev = poseidon(inputs, prev, 2);
    }
  }

  const res = poseidon.F.e(
    '18983088820287088885850106087039471251611359596827931776044660470697434019038',
  );
  // console.log('res', res);
  // console.log('eq', poseidon.F.eq(res, prev[1]));

  return poseidon.F.toString(prev[1]);
}

function toLittleEndian(value: bigint): bigint {
  value =
    ((value &
      BigInt(
        '0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00',
      )) >>
      BigInt(8)) |
    ((value &
      BigInt(
        '0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF',
      )) <<
      BigInt(8));
  value =
    ((value &
      BigInt(
        '0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000',
      )) >>
      BigInt(16)) |
    ((value &
      BigInt(
        '0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF',
      )) <<
      BigInt(16));
  value =
    ((value &
      BigInt(
        '0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000',
      )) >>
      BigInt(32)) |
    ((value &
      BigInt(
        '0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF',
      )) <<
      BigInt(32));
  value =
    ((value &
      BigInt(
        '0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000',
      )) >>
      BigInt(64)) |
    ((value &
      BigInt(
        '0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF',
      )) <<
      BigInt(64));
  value = (value >> BigInt(128)) | (value << BigInt(128));
  return value;
}

function getFirst253Bits(arr: Uint8Array): string {
  if (arr.length !== 32) {
    throw new Error('Input array must be exactly 32 bytes long');
  }

  // Create a new Uint8Array of 32 bytes to hold the first 31 bytes and the modified last byte
  const bytes = new Uint8Array(32);

  // Copy the first 31 bytes
  bytes.set(arr.slice(0, 31));

  // Get the first 5 bits of the 32nd byte (last 3 bits are ignored)
  bytes[31] = arr[31] >> 3;

  // Convert the bytes to a hex string
  const hexString = Array.from(bytes)
    .map(byte => byte.toString(16).padStart(2, '0'))
    .join('');

  // Convert hex string to BigInt
  const bigInt = BigInt('0x' + hexString);

  return bigInt.toString();
}
