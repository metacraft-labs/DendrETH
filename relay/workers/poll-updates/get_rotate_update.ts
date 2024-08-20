import { PointG1 } from '@noble/bls12-381';
import { bigint_to_array, bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
import { getPoseidonInputs, toLittleEndianBytes } from '@/utils/succinct_utils';
import { writeFileSync } from 'fs';

(async () => {
  const { DenebClient } = await import('telepathyx/src/operatorx/deneb');
  const { ChainId } = await import('telepathyx/src/operatorx/config');

  const { ssz } = await import('@lodestar/types');

  const denebClient = new DenebClient(
    'http://unstable.mainnet.beacon-api.nimbus.team/',
    ChainId.Mainnet,
  );

  const rotate = await getRotateUpdate(denebClient, ssz, 9771200);

  const outputFilePath = 'rotate.json';

  writeFileSync(outputFilePath, JSON.stringify(rotate));
})();

async function getRotateUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  let rotate = await denebClient.getRotateUpdate(slot);
  let finalizedBlock = await denebClient.getBlock(slot);

  let finalizedProposerIndexBytes = toLittleEndianBytes(
    BigInt(finalizedBlock.proposerIndex),
  );

  return {
    pubkeysBytes: rotate.nextSyncCommittee.pubkeys.map(x => Array.from(x)),
    aggregatePubkeyBytesX: Array.from(rotate.nextSyncCommittee.aggregatePubkey),
    pubkeysBigIntX: rotate.nextSyncCommittee.pubkeys
      .map(x => PointG1.fromHex(x))
      .map(x => bigint_to_array(55, 7, x.toAffine()[0].value)),
    pubkeysBigIntY: rotate.nextSyncCommittee.pubkeys
      .map(x => PointG1.fromHex(x))
      .map(x => bigint_to_array(55, 7, x.toAffine()[1].value)),
    syncCommitteeSSZ: Array.from(
      ssz.deneb.BeaconState.fields.nextSyncCommittee.hashTreeRoot(
        rotate.nextSyncCommittee,
      ),
    ),
    syncCommitteeBranch: rotate.nextSyncCommitteeBranch.map(x => Array.from(x)),
    syncCommitteePoseidon: await getPoseidonInputs(
      rotate.nextSyncCommittee.pubkeys.map(bytesToHex),
    ),
    finalizedHeaderRoot: Array.from(
      ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock),
    ),
    finalizedSlot: Array.from(
      ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(finalizedBlock.slot),
    ),
    finalizedProposerIndex: Array.from(finalizedProposerIndexBytes),
    finalizedParentRoot: Array.from(finalizedBlock.parentRoot),
    finalizedStateRoot: Array.from(finalizedBlock.stateRoot),
    finalizedBodyRoot: Array.from(
      ssz.deneb.BeaconBlockBody.hashTreeRoot(finalizedBlock.body),
    ),
  };
}
