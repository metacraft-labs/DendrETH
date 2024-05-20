import ssz from '@chainsafe/ssz';
import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { verify } from 'circom-pairing/test/index';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  getCommitmentMapperProof,
  getNthParent,
  gindexFromIndex,
} from '../../../utils/common_utils';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { write, writeFileSync } from 'fs';
import JSONbig from 'json-bigint';
import { verifyMerkleProof } from '@dendreth/utils/ts-utils/ssz-utils';

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

  console.log(bytesToHex(signing_root));

  console.log(
    await verify(formatHex(signature), signing_root, deposit_message.pubkey),
  );

  let slot = 5066944;

  let beaconApi = await getBeaconApi([
    'http://unstable.sepolia.beacon-api.nimbus.team/',
  ]);

  let { beaconState: beaconState } = await beaconApi.getBeaconState(
    BigInt(slot),
  );
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

  console.log(beaconState.balances[foundIndex]);

  let balanceIndex = Math.floor(foundIndex / 4);

  validator!.pubkey = bytesToHex(validator!.pubkey) as any;
  validator!.withdrawalCredentials = bytesToHex(
    validator!.withdrawalCredentials!,
  ) as any;

  console.log(gindexFromIndex(BigInt(foundIndex), 40n));

  console.log(
    await redis.extractHashFromCommitmentMapperProof(
      1n,
      BigInt(beaconState.slot),
      'poseidon',
    ),
  );

  console.log(beaconState.slot);
  let deposit_accumulator_input = {
    validator: validator,
    validatorDeposit: {
      pubkey,
      depositIndex: deposit_index.toString(),
      signature,
      depositMessageRoot: deposit_message_hash_tree_root,
    },
    commitmentMapperRoot: await redis.extractHashFromCommitmentMapperProof(
      1n,
      BigInt(beaconState.slot),
      'poseidon',
    ),
    commitmentMapperProof: await getCommitmentMapperProof(
      BigInt(beaconState.slot),
      gindexFromIndex(BigInt(foundIndex), 40n),
      'poseidon',
      redis,
    ),
    validatorIndex: foundIndex,
    // validatorDepositRoot: // TODO: we should have the deposits commitment mapper root
    // validatorDepositProof: // TODO: we should have the deposits commitment mapper proof
    balanceTreeRoot: bytesToHex(
      balancesTree.getNode(
        getNthParent(balanceZeroGindex + BigInt(balanceIndex), 22n),
      ).root,
    ),
    balanceLeaf: bytesToHex(
      balancesTree.getNode(balanceZeroGindex + BigInt(balanceIndex)).root,
    ),
    balanceProof: balancesTree
      .getSingleProof(balanceZeroGindex + BigInt(balanceIndex))
      .slice(0, 22)
      .map(bytesToHex),
    blsSignatureProofKey: `bls12_381_${pubkey}_${deposit_index}`,
    currentEpoch: (BigInt(beaconState.slot) / 32n).toString(),
    isDummy: false,
    eth1DepositIndex: beaconState.eth1DepositIndex,
  };

  let result = verifyMerkleProof(
    [
      'cbf1a3690b000000798608680b000000cd73407d0b00000001ad6a6b0b000000',
      '34f735cad9ae2d061fbab0682064d1b37e8c227e0f13e07457ce12d69e97da43',
      'efb80785674ab41400abe50d7b3b837128ac54451ae0bf433cb9e4d9cbfc6c4c',
      'c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c',
      '536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c',
      '9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30',
      'd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1',
      '87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c',
      '26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193',
      '506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1',
      'ffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b',
      '6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220',
      'b7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f',
      'df6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e',
      'b58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784',
      'd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb',
      '8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb',
      '8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab',
      '95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4',
      'f893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f',
      'cddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa',
      '8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c',
    ],
    bytesToHex(
      balancesTree.getNode(
        getNthParent(balanceZeroGindex + BigInt(balanceIndex), 22n),
      ).root,
    ),
    'b07ad63907000000045d8b6d0b000000be642c690b0000001cba346c0b000000',
    0n,
  );

  console.log(result);

  let deposit_accumulator_string = JSONbig.stringify(deposit_accumulator_input);

  writeFileSync('deposit_accumulator_input.json', deposit_accumulator_string);
}
