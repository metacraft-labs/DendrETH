import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
  utils,
} from '../../../../libs/typescript/ts-utils/bls';
import { ssz } from '@lodestar/types';
import { writeFileSync } from 'fs';
import { BitVectorType } from '@chainsafe/ssz';
import * as path from 'path';
import { getFilesInDir } from '../../../../libs/typescript/ts-utils/data';

const hashToField = utils.hashToField;

function getMessage(blockRoot: Uint8Array) {
  const genesisValidatorsRoot: Uint8Array = hexToBytes(
    '0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95',
  );

  const ForkData = ssz.phase0.ForkData;
  let fork_data = ForkData.defaultValue();
  fork_data.currentVersion = hexToBytes('0x01000000');

  fork_data.genesisValidatorsRoot = genesisValidatorsRoot;
  let fork_data_root = ForkData.hashTreeRoot(fork_data);

  let domain = new Uint8Array(32);
  const DOMAIN_SYNC_COMMITTEE = hexToBytes('0x07000000');
  for (let i = 0; i < 4; i++) domain[i] = DOMAIN_SYNC_COMMITTEE[i];
  for (let i = 0; i < 28; i++) domain[i + 4] = fork_data_root[i];

  const SigningData = ssz.phase0.SigningData;
  let signing_data = SigningData.defaultValue();
  signing_data.objectRoot = blockRoot;
  signing_data.domain = domain;
  return SigningData.hashTreeRoot(signing_data);
}

async function getProof(prevUpdate, update) {
  let points: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x =>
    PointG1.fromHex(x.slice(2)),
  );
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(
    update.sync_aggregate.sync_committee_bits,
  );
  let signature: PointG2 = PointG2.fromSignature(
    formatHex(update.sync_aggregate.sync_committee_signature),
  );
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
  let message = getMessage(hash);
  let u = await hashToField(message, 2);

  let input = {
    points: points.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
    aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey)
      .toString(2)
      .split(''),
    bitmask: bitmask.toBoolArray().map(x => (x ? '1' : '0')),
    signature: [
      [
        bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[0].c1.value),
      ],
      [
        bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[1].c1.value),
      ],
    ],
    hash: [
      [bigint_to_array(55, 7, u[0][0]), bigint_to_array(55, 7, u[0][1])],
      [bigint_to_array(55, 7, u[1][0]), bigint_to_array(55, 7, u[1][1])],
    ],
  };

  return input;
}

(async () => {
  const UPDATES = getFilesInDir(
    path.join(__dirname, '../../../', 'data', 'mainnet', 'updates'),
  );

  let prevUpdate = UPDATES[0];

  for (let update of UPDATES.slice(1, 2)) {
    writeFileSync(
      path.join(__dirname, 'input.json'),
      JSON.stringify(
        await getProof(
          JSON.parse(prevUpdate.toString()),
          JSON.parse(update as unknown as string),
        ),
      ),
    );
  }
})();
