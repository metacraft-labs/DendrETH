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
  let GENESIS_FORK_VERSION = '0x90000069';
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
      '0xb781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73',
    ),
    withdrawalCredentials: hexToBytes(
      '0x00ea361fd66a174b289f0b26ed7bbcaabdec4b7d47d5527ff72a994c9c1c156f',
    ),
    amount: 32000000000,
  };

  let signature =
    '0xb735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275';

  let deposit_message_hash_tree_root =
    ssz.phase0.DepositMessage.hashTreeRoot(deposit_message);

  let domain =
    formatHex(DOMAIN_DEPOSIT) + formatHex(fork_data_root.slice(0, 56));

  let signing_root = ssz.phase0.SigningData.hashTreeRoot({
    objectRoot: deposit_message_hash_tree_root,
    domain: hexToBytes(domain),
  });

  console.log(
    await verify(formatHex(signature), signing_root, deposit_message.pubkey),
  );

  let slot = 5059552;

  let beaconApi = await getBeaconApi([
    'http://unstable.sepolia.beacon-api.nimbus.team/',
  ]);

  let { beaconState: beaconState } = await beaconApi.getBeaconState(5046663n);
  beaconState.balances = beaconState.balances.slice(1573, 1573 + 32);
  beaconState.validators = beaconState.validators.slice(1573, 1573 + 32);

  console.log(beaconState.validators.length);

  const validatorsView = ssz.deneb.BeaconState.fields.validators.toViewDU(
    beaconState.validators,
  );
  const validatorTree = new Tree(validatorsView.node);

  console.log(bytesToHex(validatorTree.getNode(2n).root));

  console.log(
    bytesToHex(
      ssz.deneb.BeaconState.fields.validators.hashTreeRoot(
        beaconState.validators,
      ),
    ),
  );

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

  console.log(gindexFromIndex(BigInt(foundIndex), 40n));

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
    eth1DepositIndex: beaconState.eth1DepositIndex,
  };

  console.log(deposit_accumulator_input);
}
