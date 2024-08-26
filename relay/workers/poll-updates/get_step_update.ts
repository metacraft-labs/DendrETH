import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import {
  hexToBits,
  reverseEndianness,
} from '@dendreth/utils/ts-utils/hex-utils';
import { sha256 } from 'ethers/lib/utils';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
import {
  getPoseidonInputs,
  toLittleEndianBytes,
} from '../../utils/telepathy_utils';
import { writeFileSync } from 'fs';

(async () => {
  const { DenebClient } = await import('telepathyx/src/operatorx/deneb');
  const { ChainId } = await import('telepathyx/src/operatorx/config');

  const { ssz } = await import('@lodestar/types');

  const denebClient = new DenebClient(
    'http://unstable.sepolia.beacon-api.nimbus.team',
    ChainId.Sepolia,
  );

  const step = await getStepUpdate(denebClient, ssz, 5239744);

  writeFileSync('step.json', JSON.stringify(step));
})();

async function getStepUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  const step = await denebClient.getStepUpdate(slot);

  let { domain, signing_root } = calcDomainAndSigningRoot(
    step.forkVersion,
    step.genesisValidatorsRoot,
    step.attestedHeaderRoot,
  );

  let aggregationBits: number[] = step.syncAggregate.syncCommitteeBits
    .toBoolArray()
    .map(x => (x ? 1 : 0));
  let participation = aggregationBits.reduce((a, b) => a + b, 0);

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

  let syncCommitteePoseidon = await getPoseidonInputs(
    step.currentSyncCommittee.pubkeys.map(bytesToHex),
  );

  return {
    attestedHeaderRoot: Array.from(step.attestedHeaderRoot),
    attestedSlot: Array.from(
      toLittleEndianBytes(BigInt(step.attestedBlock.slot)),
    ),
    attestedProposerIndex: Array.from(
      toLittleEndianBytes(BigInt(step.attestedBlock.proposerIndex)),
    ),
    attestedParentRoot: Array.from(step.attestedBlock.parentRoot),
    attestedStateRoot: Array.from(step.attestedBlock.stateRoot),
    attestedBodyRoot: Array.from(
      ssz.deneb.BeaconBlockBody.hashTreeRoot(step.attestedBlock.body as any),
    ),
    finalizedHeaderRoot: Array.from(step.finalizedHeaderRoot),
    finalizedSlot: Array.from(
      toLittleEndianBytes(BigInt(step.finalizedBlock.slot)),
    ),
    finalizedProposerIndex: Array.from(
      toLittleEndianBytes(BigInt(step.finalizedBlock.proposerIndex)),
    ),
    finalizedParentRoot: Array.from(step.finalizedBlock.parentRoot),
    finalizedStateRoot: Array.from(step.finalizedBlock.stateRoot),
    finalizedBodyRoot: Array.from(
      ssz.deneb.BeaconBlockBody.hashTreeRoot(step.finalizedBlock.body as any),
    ),
    pubkeysX: step.currentSyncCommittee.pubkeys
      .map(x => PointG1.fromHex(x))
      .map(x => bigint_to_array(55, 7, x.toAffine()[0].value)),
    pubkeysY: step.currentSyncCommittee.pubkeys
      .map(x => PointG1.fromHex(x))
      .map(x => bigint_to_array(55, 7, x.toAffine()[1].value)),
    aggregationBits,
    signature,
    domain: Array.from(hexToBytes(domain)),
    signingRoot: Array.from(hexToBytes(signing_root)),
    executionStateRoot: Array.from(step.executionStateRoot),
    participation: participation,
    syncCommitteePoseidon: syncCommitteePoseidon,
    finalityBranch: step.finalityBranch.map(x => Array.from(x)),
    executionStateBranch: step.executionStateBranch.map(x => Array.from(x)),
    publicInputsRoot: getPublicInputsRoot(
      toLittleEndianBytes(BigInt(step.attestedBlock.slot)),
      toLittleEndianBytes(BigInt(step.finalizedBlock.slot)),
      step.finalizedHeaderRoot,
      participation,
      step.executionStateRoot,
      reverseEndianness(
        BigInt(syncCommitteePoseidon).toString(16).padStart(64, '0'),
      ),
    ).toString(),
  };
}

function calcDomainAndSigningRoot(
  forkVersion: Uint8Array,
  genesisValidatorsRoot: Uint8Array,
  attestedHeaderRoot: Uint8Array,
) {
  let sha256_fork_version = sha256(
    '0x' +
      bytesToHex(forkVersion).padEnd(64, '0') +
      bytesToHex(genesisValidatorsRoot),
  );

  const DOMAIN_SYNC_COMMITTEE = '0x07000000';
  let domain =
    formatHex(DOMAIN_SYNC_COMMITTEE) +
    formatHex(sha256_fork_version).slice(0, 56);
  let signing_root = sha256('0x' + bytesToHex(attestedHeaderRoot) + domain);
  return { domain, signing_root };
}

function getPublicInputsRoot(
  attestedSlotBytes: Uint8Array,
  finalizedSlotBytes: Uint8Array,
  finalizedHeaderRoot: Uint8Array,
  participation: number,
  executionStateRoot: Uint8Array,
  syncCommitteePoseidonHex: string,
) {
  let sha0 = sha256(
    '0x' + bytesToHex(attestedSlotBytes) + bytesToHex(finalizedSlotBytes),
  );
  let sha1 = sha256(sha0 + bytesToHex(finalizedHeaderRoot));
  let participationHex = reverseEndianness(
    BigInt(participation).toString(16).padStart(64, '0'),
  );
  let sha2 = sha256(sha1 + participationHex);
  let sha3 = sha256(sha2 + bytesToHex(executionStateRoot));
  let sha4 = sha256(sha3 + syncCommitteePoseidonHex);
  let bits = hexToBits(sha4);
  let publicInputsRootBits = bits
    .reduce(
      (acc, _, i) => (i % 8 === 0 ? acc.push(bits.slice(i, i + 8)) : acc, acc),
      [] as number[][],
    )
    .flatMap(x => x.reverse())
    .slice(0, 253);

  return BigInt('0b' + publicInputsRootBits.reverse().join(''));
}
