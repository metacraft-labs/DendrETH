import { init, PublicKey, Signature } from "@chainsafe/bls";
import { BitVectorType } from "@chainsafe/ssz";
import { PointG1 } from "@noble/bls12-381";
import { bytesToHex, hexToBytes } from "../../ts-utils/bls";
import { ssz } from "@chainsafe/lodestar-types";
import * as update1 from "../../data/mainnet/updates/00290.json";
import * as update2 from "../../data/mainnet/updates/00291.json";

function getMessage(blockRoot: Uint8Array) {
  const genesisValidatorsRoot: Uint8Array = hexToBytes(
    "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"
  );

  const ForkData = ssz.phase0.ForkData;
  let fork_data = ForkData.defaultValue();
  fork_data.currentVersion = hexToBytes("0x01000000");

  fork_data.genesisValidatorsRoot = genesisValidatorsRoot;
  let fork_data_root = ForkData.hashTreeRoot(fork_data);

  let domain = new Uint8Array(32);
  const DOMAIN_SYNC_COMMITTEE = hexToBytes("0x07000000");
  for (let i = 0; i < 4; i++) domain[i] = DOMAIN_SYNC_COMMITTEE[i];
  for (let i = 0; i < 28; i++) domain[i + 4] = fork_data_root[i];

  console.log("DOMAIN ", domain.join());

  const SigningData = ssz.phase0.SigningData;
  let signing_data = SigningData.defaultValue();
  signing_data.objectRoot = blockRoot;
  signing_data.domain = domain;
  return SigningData.hashTreeRoot(signing_data);
}

(async () => {
  let points: PointG1[] = update1.next_sync_committee.pubkeys.map(x => PointG1.fromHex(x.slice(2)));
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(update2.sync_aggregate.sync_committee_bits);

  let sum = points.filter((_, i) => bitmask.get(i)).reduce((prev, curr) => prev.add(curr));

  let publicKey = sum.toHex(true);
  console.log("Public key: ", publicKey);
  let signature = update2.sync_aggregate.sync_committee_signature;


  const BeaconBlockHeader = ssz.phase0.BeaconBlockHeader;
  let block_header = BeaconBlockHeader.defaultValue();
  block_header.slot = Number.parseInt(update2.attested_header.slot);
  block_header.proposerIndex = Number.parseInt(update2.attested_header.proposer_index);
  block_header.parentRoot = hexToBytes(update2.attested_header.parent_root);
  block_header.stateRoot = hexToBytes(update2.attested_header.state_root);
  block_header.bodyRoot = hexToBytes(update2.attested_header.body_root);
  let hash = BeaconBlockHeader.hashTreeRoot(block_header);
  console.log("Block root:", bytesToHex(hash));
  let message = getMessage(hash);
  console.log("Message: ", bytesToHex(message));

  await init("herumi");
  let p = PublicKey.fromHex(publicKey);

  let s = Signature.fromHex(signature);

  console.log("Is valid: ", s.verify(p, message));
})();
