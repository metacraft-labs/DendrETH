import ssz from '@chainsafe/ssz';
import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { verify } from '../../../vendor/circom-pairing/test/index';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  getCommitmentMapperProof,
  gindexFromIndex,
} from '../validators_commitment_mapper_tree/utils';
import { Tree } from '@chainsafe/persistent-merkle-tree';

(async () => {
  const { ssz } = await import('@lodestar/types');

  let DOMAIN_DEPOSIT = '0x03000000';
  let GENESIS_FORK_VERSION = '0x00000000';
  let genesis_validator_root =
    '0x0000000000000000000000000000000000000000000000000000000000000000';

  let fork_data_root = bytesToHex(
    ssz.phase0.ForkData.hashTreeRoot({
      currentVersion: hexToBytes(GENESIS_FORK_VERSION),
      genesisValidatorsRoot: hexToBytes(genesis_validator_root),
    }),
  );

  let deposit_message = {
    pubkey: hexToBytes(
      '0x90823dc2e5ab8a52a0b32883ea8451cbe4c921a42ce439f4fb306a90e9f267e463241da7274b6d44c2e4b95ddbcb0ad3',
    ),
    withdrawalCredentials: hexToBytes(
      '0x005bfe00d82068a0c2a6687afaf969dad5a9c663cb492815a65d203885aaf993',
    ),
    amount: 32000000000,
  };

  let signature =
    '0x802899068eb4b37c95d46869947cac42b9c65b90fcb3fde3854c93ad5737800c01e9c82e174c8ed5cc18210bd60a94ea0082a850817b1dddd4096059b6846417b05094c59d3dd7f4028ed9dff395755f9905a88015b0ed200a7ec1ed60c24922';

  let deposit_message_hash_tree_root =
    ssz.phase0.DepositMessage.hashTreeRoot(deposit_message);

  let domain =
    formatHex(DOMAIN_DEPOSIT) + formatHex(fork_data_root.slice(0, 56));

  let signing_root = ssz.phase0.SigningData.hashTreeRoot({
    objectRoot: deposit_message_hash_tree_root,
    domain: hexToBytes(domain),
  });

  let slot = 5046663;

  let beaconApi = await getBeaconApi([
    'http://unstable.sepolia.beacon-api.nimbus.team/',
  ]);

  let { beaconState: beaconState } = await beaconApi.getBeaconState(5046663n);

  let redis = new Redis('localhost', 6379);

  generate_leaf_level_data(
    bytesToHex(deposit_message.pubkey),
    1n,
    signature,
    bytesToHex(deposit_message_hash_tree_root),
    beaconState,
    redis,
  );
})();

type BeaconState = Awaited<
  ReturnType<BeaconApi['getBeaconState']>
>['beaconState'];

async function generate_leaf_level_data(
  pubkey: string,
  deposit_index: bigint,
  signature: string,
  deposit_message_hash_tree_root: string,
  beaconState: BeaconState,
  redis: Redis,
) {
  const { ssz } = await import('@lodestar/types');

  let foundIndex = -1;
  let validator = beaconState.validators.find((validator, i) => {
    if (formatHex(bytesToHex(validator.pubkey)) === formatHex(pubkey)) {
      foundIndex = i;
      return true;
    }
    return false;
  });

  if (foundIndex === -1) {
    throw new Error('Validator not found');
  }

  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );
  const balancesTree = new Tree(balancesView.node);
  const balanceZeroGindex = ssz.deneb.BeaconState.fields.balances.getPathInfo([
    0,
  ]).gindex;

  let balanceIndex = Math.floor(foundIndex / 4);

  validator!.pubkey = bytesToHex(validator!.pubkey) as any;
  validator!.withdrawalCredentials = bytesToHex(
    validator!.withdrawalCredentials!,
  ) as any;

  let deposit_accumulator_input = {
    validator: validator,
    validatorDeposit: {
      pubkey,
      deposit_index,
      signature,
      deposit_message_hash_tree_root,
    },
    commitmentMapperHashTreeRoot:
      await redis.extractHashFromCommitmentMapperProof(
        1n,
        BigInt(beaconState.slot),
        'poseidon',
      ),
    commimtnetMapperProof: await getCommitmentMapperProof(
      BigInt(beaconState.slot),
      gindexFromIndex(BigInt(foundIndex), 40n),
      'poseidon',
      redis,
    ),
    validatorIndex: foundIndex,
    // validatorDepositRoot: // TODO: we should have the deposits commitment mapper root
    // validatorDepositProof: // TODO: we should have the deposits commitment mapper proof
    balance_tree_root: bytesToHex(
      ssz.deneb.BeaconState.fields.balances.hashTreeRoot(beaconState.balances),
    ),
    balance_leaf: bytesToHex(
      balancesTree.getNode(balanceZeroGindex + BigInt(balanceIndex)).root,
    ),
    balance_proof: balancesTree
      .getSingleProof(balanceZeroGindex + BigInt(balanceIndex))
      .slice(0, 22)
      .map(bytesToHex),
    blsSignatureProofKey: `bls12_381_${pubkey}_${deposit_index}`,
    currentEpoch: BigInt(beaconState.slot) / 32n,
    isDummy: false,
    eth1DepositIndex: deposit_index,
  };

  console.log(deposit_accumulator_input);
}
