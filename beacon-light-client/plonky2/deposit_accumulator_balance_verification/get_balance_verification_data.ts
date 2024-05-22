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
      '0x8fd1defb5dc823f93ba4e42046e52c61c3b46cdd473a8ae0d743bad8aebf85134f20b794d41125778485eb576d9a5b7a',
    ),
    withdrawalCredentials: hexToBytes(
      '0x0100000000000000000000000b18ddbc066ee097871d4973c2fc47131a18a07a',
    ),
    amount: 32000000000,
  };

  let signature =
    '0x8b8d80e8f19b8e6d40687e8a99d9f1135efa2deedf49d7268e8b424d4075b85806d3a664873360d494ce6040bba3f4ca0fe8a89e1d9d67c5ba61f028ddce14453fc183c0960bd0497084235ef008790aa5b5d75f020616cf64418deb15b7ad42';

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
  const {ssz} = await import('@lodestar/types');

  let foundIndex = -1;
  let validator = beaconState.validators.find((validator, i) => {
    if (formatHex(bytesToHex(validator.pubkey)) === formatHex(pubkey)) {
      foundIndex = i;
      return true;
    }
    return false;
  });


  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );
  const balancesTree = new Tree(balancesView.node);
  const balanceZeroGindex = ssz.deneb.BeaconState.fields.balances.getPathInfo([0]).gindex;

  let balanceIndex = Math.floor(foundIndex / 4);

  let deposit_accumulator_input = {
    validator: validator,
    validatorDeposit: {
      pubkey,
      deposit_index,
      signature,
      deposit_message_hash_tree_root,
    },
    commitmentMapperHashTreeRoot:
      await redis.extractHashFromCommitmentMapperProof(1n, BigInt(beaconState.slot), 'poseidon'),
    commimtnetMapperProof: await getCommitmentMapperProof(
      BigInt(beaconState.slot),
      gindexFromIndex(BigInt(foundIndex), 40n),
      'poseidon',
      redis,
    ),
    validatorIndex: foundIndex,
    validatorDepositRoot: // TODO: we should have the deposits commitment mapper root
    validatorDepositProof: // TODO: we should have the deposits commitment mapper proof
    balance_tree_root: ssz.deneb.BeaconState.fields.balances.hashTreeRoot(beaconState.balances),
    balance_leaf: balancesTree.getNode(balanceZeroGindex + BigInt(balanceIndex)).root,
    balance_proof: balancesTree.getSingleProof(balanceZeroGindex + BigInt(balanceIndex)).slice(0, 22).map(bytesToHex),
    blsSignatureProofKey: `bls12_381_${pubkey}_${deposit_index}`,
    currentEpoch: BigInt(beaconState.slot) / 32n,
    isDummy: false,
    eth1DepositIndex: deposit_index,
  };
}
