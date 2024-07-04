import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { BitVectorType } from '@chainsafe/ssz';
import { sha256 } from 'ethers/lib/utils';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
const fs = require('fs');

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

  // let syncCommitteePoseidon = ''; // TODO: find some way

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
  let aggregationBits = SyncCommitteeBits.fromJson(
    step.syncAggregate.syncCommitteeBits,
  )
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
    bytesToHex(step.forkVersion) + bytesToHex(step.genesisValidatorsRoot),
  );

  let domain =
    'config.DOMAIN_SYNC_COMMITTEE' + sha256_fork_version.slice(2, 58);

  let signing_root = sha256(bytesToHex(step.attestedHeaderRoot) + domain);

  let participation = aggregationBits
    .map(x => Number(x))
    .reduce((a, b) => a + b, 0);

  let syncCommitteePoseidon = ''; // TODO: read from contract

  // Compute the necessary values in the correct format
  let finalityBranch = step.finalityBranch;
  let executionStateRoot = step.executionStateRoot;
  let executionStateBranch = step.executionStateBranch;

  let attestedHeaderRoot = step.attestedHeaderRoot;
  let attestedSlot = step.attestedBlock.slot;
  let attestedProposerIndex = step.attestedBlock.proposerIndex;
  let attestedParentRoot = step.attestedBlock.parentRoot;
  let attestedStateRoot = step.attestedBlock.stateRoot;
  let attestedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.attestedBlock.body as any,
  );

  let finalizedHeaderRoot = step.finalizedHeaderRoot;
  let finalizedSlot = step.finalizedBlock.slot;
  let finalizedProposerIndex = step.finalizedBlock.proposerIndex;
  let finalizedParentRoot = step.finalizedBlock.parentRoot;
  let finalizedStateRoot = step.finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.finalizedBlock.body as any,
  );

  // Create the JSON object in the specified order
  const jsonOutput = {
    /* Attested Header */
    attestedHeaderRoot,
    attestedSlot,
    attestedProposerIndex,
    attestedParentRoot,
    attestedStateRoot,
    attestedBodyRoot,

    /* Finalized Header */
    finalizedHeaderRoot,
    finalizedSlot,
    finalizedProposerIndex,
    finalizedParentRoot,
    finalizedStateRoot,
    finalizedBodyRoot,

    /* Sync Committee Protocol */
    pubkeysX,
    pubkeysY,
    aggregationBits,
    signature,
    domain,
    signingRoot: signing_root,
    participation,
    syncCommitteePoseidon,

    /* Finality Proof */
    finalityBranch,

    /* Execution State Proof */
    executionStateRoot,
    executionStateBranch,

    /* Commitment to Public Inputs */
    publicInputsRoot: '', // TODO
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

  let syncCommitteePoseidon = ''; // TODO: find some way

  let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
  let finalizedSlot = ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(
    finalizedBlock.slot,
  );
  let finalizedProposerIndex = finalizedBlock.proposerIndex;
  let finalizedParentRoot = finalizedBlock.parentRoot;
  let finalizedStateRoot = finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    finalizedBlock.body as any,
  );
  const jsonOutput = {
    pubkeysBytes,
    aggregatePubkeyBytesX,
    pubkeysBigIntX,
    pubkeysBigIntY,
    syncCommitteeSSZ,
    syncCommitteeBranch,
    syncCommitteePoseidon,
    finalizedHeaderRoot,
    finalizedSlot,
    finalizedProposerIndex,
    finalizedParentRoot,
    finalizedStateRoot,
    finalizedBodyRoot,
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
