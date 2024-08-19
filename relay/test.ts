import { PointG1, PointG2, verify } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
  utils,
} from '@dendreth/utils/ts-utils/bls';
import { hexToBits, reverseEndianness } from '@dendreth/utils/ts-utils/hex-utils';
import { BitVectorType } from '@chainsafe/ssz';
import { sha256 } from 'ethers/lib/utils';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
const fs = require('fs');

import { buildPoseidon, buildPoseidonReference } from 'circomlibjs';

import { numberToBytesBE } from '@noble/bls12-381/math';
import { exit } from 'process';

import { bitsToHex } from '@dendreth/utils/ts-utils/hex-utils';
(async () => {
  const { DenebClient } = await import('telepathyx/src/operatorx/deneb');
  const { ChainId } = await import('telepathyx/src/operatorx/config');

  const { ssz } = await import('@lodestar/types');

  const denebClient = new DenebClient(
    'http://unstable.sepolia.beacon-api.nimbus.team',
    ChainId.Sepolia,
  );

  await getStepUpdate(denebClient, ssz, 5239744);
  // await getRotateUpdate(denebClient, ssz, 5239744);

  // let rotate = await denebClient.getRotateUpdate(5239744);
  let finalizedBlock = await denebClient.getBlock(5239744);

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

  let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
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
  let pubkeysBytes = step.currentSyncCommittee.pubkeys;

  let pubkeysX = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  let pubkeysY = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));

  let points = step.currentSyncCommittee.pubkeys.map(x => PointG1.fromHex(x));

  let agg = PointG1.ZERO;

  for (let i = 0; i < points.length; i++) {
    agg = agg.add(points[i]);
  }

  let aggX = bigint_to_array(55, 7, agg.toAffine()[0].value);
  let aggY = bigint_to_array(55, 7, agg.toAffine()[1].value);

  console.log('aggX', aggX);
  console.log('aggY', aggY);

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

  console.log('Fork version', step.forkVersion);

  let sha256_fork_version = sha256(
    '0x' +
    bytesToHex(step.forkVersion).padEnd(64, '0') +
    bytesToHex(step.genesisValidatorsRoot),
  );

  const DOMAIN_SYNC_COMMITTEE = '0x07000000';
  let domain = formatHex(DOMAIN_SYNC_COMMITTEE) + formatHex(sha256_fork_version).slice(0, 56);
  let signing_root = sha256(
    "0x" + bytesToHex(step.attestedHeaderRoot) + domain,
  );

  let aggHex = agg.toHex(true);

  console.log("Params", bytesToHex(step.syncAggregate.syncCommitteeSignature), signing_root, aggHex);

  const result = await verify(bytesToHex(step.syncAggregate.syncCommitteeSignature), formatHex(signing_root), aggHex);

  console.log("Verify bls result", result);

  let participation = aggregationBits
    .map(x => Number(x))
    .reduce((a, b) => a + b, 0);
  let syncCommitteePoseidon = await getPoseidonInputs(pubkeysBytes);
  let syncCommitteePoseidonInHex = reverseEndianness(BigInt(syncCommitteePoseidon).toString(16));
  console.log("Sync Committee Bytes", hexToBytes(syncCommitteePoseidonInHex));
  // let syncCommitteePoseidonBytes = hexToBytes(syncCommitteePoseidonInHex);

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
  console.log("Participation:", BigInt(participation));
  // console.log('Participation little endian:', toLittleEndian(BigInt(participation)));
  let participationHex = reverseEndianness(BigInt(participation).toString(16).padStart(64, '0'));

  console.log('participation bytes', participationHex);

  let sha2 = sha256(sha1 + participationHex);

  console.log('Sha2', hexToBytes(sha2));

  let sha3 = sha256(sha2 + bytesToHex(executionStateRoot));
  let sha4 = sha256(sha3 + syncCommitteePoseidonInHex);

  console.log('sha4', sha4);

  let bits = hexToBits(sha4);

  let publicInputsRoot = hexToBits(sha4).reduce((acc, _, i) => (i % 8 === 0 ? acc.push(bits.slice(i, i + 8)) : acc, acc), [] as number[][]).flatMap(x => x.reverse()).slice(0, 253);

  let publicInputRootNumber = BigInt('0b' + publicInputsRoot.reverse().join(''));
  const bit_array = [
    0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0,
    1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1,
    0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1,
    0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0,
    0, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1,
    1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1,
    0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 1,
    1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1,
    1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1,
    1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0,
    0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0
  ]
  // Create the JSON object in the specified order
  const jsonOutput = {
    // /* Attested Header */
    // attestedHeaderRoot: Array.from(Buffer.from(attestedHeaderRoot)),
    // attestedSlot: Array.from(Buffer.from(attestedSlotBytes)),
    // attestedProposerIndex: Array.from(Buffer.from(attestedProposerIndexBytes)),
    // attestedParentRoot: Array.from(Buffer.from(attestedParentRoot)),
    // attestedStateRoot: Array.from(Buffer.from(attestedStateRoot)),
    // attestedBodyRoot: Array.from(Buffer.from(attestedBodyRoot)),
    // /* Finalized Header */
    // finalizedHeaderRoot: Array.from(Buffer.from(finalizedHeaderRoot)),
    // finalizedSlot: Array.from(Buffer.from(finalizedSlotBytes)),
    // finalizedProposerIndex: Array.from(
    //   Buffer.from(finalizedProposerIndexBytes),
    // ),
    // finalizedParentRoot: Array.from(Buffer.from(finalizedParentRoot)),
    // finalizedStateRoot: Array.from(Buffer.from(finalizedStateRoot)),
    // finalizedBodyRoot: Array.from(Buffer.from(finalizedBodyRoot)),
    /* Sync Committee Protocol */
    pubkeysX,
    pubkeysY,
    aggregationBits,
    signature,
    // domain: Array.from(Buffer.from(hexToBytes(domain))),
    signingRoot: Array.from(Buffer.from(hexToBytes(signing_root))),
    // executionStateRoot: Array.from(Buffer.from(executionStateRoot)),
    // participation: participation,
    syncCommitteePoseidon: syncCommitteePoseidon,
    /* Finality Proof */
    // finalityBranch: finalityBranch.map(finalityB =>
    //   Array.from(Buffer.from(finalityB)),
    // ),
    /* Execution State Proof */
    // executionStateBranch: executionStateBranch.map(
    //   executionSB => Array.from(Buffer.from(executionSB)), //9 not 8
    // ),
    // /* Commitment to Public Inputs */
    // publicInputsRoot: publicInputRootNumber.toString(),
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

  let syncCommitteePoseidon = await getPoseidonInputs(pubkeysBytes);

  // let syncCommitteePoseidonInHex = BigInt(syncCommitteePoseidon).toString(16);
  // let syncCommitteePoseidonBytes = hexToBytes(syncCommitteePoseidonInHex);

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
    syncCommitteePoseidon,
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

async function getPoseidonInputs(pubkeysBytes: any) {
  let pubkeys = pubkeysBytes.map(pKeys =>
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

  let poseidon = await buildPoseidonReference();

  let poseidonValFlat: string[] = [];
  for (let i = 0; i < 512; i++) {
    for (let j = 0; j < 7; j++)
      for (let l = 0; l < 2; l++) {
        poseidonValFlat[i * 7 * 2 + j * 2 + l] = pubKeysArrayStr[i][l][j];
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

  // const res = poseidon.F.e(
  //   '18983088820287088885850106087039471251611359596827931776044660470697434019038',
  // );
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
  console.log("arr", arr);
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

// // 1
// attestedSlot+
// finalizedSlot+
// finalizedHeaderRoot+
// executionStateRoot+
// participation
// syncCommitteePoseidon+

// publicInputsRoot


// // 3.5
// pubkeysX+
// pubkeysY+
// aggregationBits
// signature
// signingRoot+
// syncCommitteePoseidon+

// // 4
// finalizedHeaderRoot+
// finalityBranch
// attestedStateRoot+



