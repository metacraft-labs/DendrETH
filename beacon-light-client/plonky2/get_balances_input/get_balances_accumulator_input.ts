import yargs from 'yargs';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import config from '../common_config.json';
import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import CONSTANTS from '../constants/validator_commitment_constants.json';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { hexToBits } from '../../../libs/typescript/ts-utils/hex-utils';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { computeEpochAt } from '../../../libs/typescript/ts-utils/ssz-utils';
const CIRCUIT_SIZE = 2;
let TAKE: number | undefined;

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .option('redis-host ', {
      alias: 'redis-host',
      describe: 'The Redis host',
      type: 'string',
      default: config['redis-host'],
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: Number(config['redis-port']),
      description: 'Sets a custom redis connection',
    })
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: config['beacon-node'],
      description: 'Sets a custom beacon node url',
    })
    .option('slot', {
      alias: 'slot',
      describe: 'The state slot',
      type: 'number',
      default: undefined,
      description: 'Fetches the balances for this slot',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: undefined,
      description: 'Sets the number of validators to take',
    })
    .options('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  TAKE = options['take'];

  const first_level_queue = new WorkQueue(
    new KeyPrefix(`${CONSTANTS.balanceVerificationAccumulatorProofKey}:0`),
  );

  const beaconApi = new BeaconApi([options['beacon-node']]);
  const slot =
    options['slot'] !== undefined
      ? options['slot']
      : Number(await beaconApi.getHeadSlot());

  const { beaconState } = await beaconApi.getBeaconState(slot);

  const offset = Number(options['offset']) || 0;
  const take = TAKE !== undefined ? TAKE + offset : undefined;
  // const validators = beaconState.validators.slice(offset, take);
  // beaconState.balances = beaconState.balances.slice(offset, take);
  // beaconState.validators = validators;

  // const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
  //   beaconState.balances,
  // );

  // const balancesTree = new Tree(balancesView.node);

  // const balanceZeroIndex = ssz.capella.BeaconState.fields.balances.getPathInfo([
  //   0,
  // ]).gindex;

  // const balances: number[][] = [];

  // for (let i = 0; i < TAKE / 4; i++) {
  //   balances.push(
  //     hexToBits(
  //       bytesToHex(balancesTree.getNode(balanceZeroIndex + BigInt(i)).root),
  //     ),
  //   );
  // }

  // if (balances.length % (CIRCUIT_SIZE / 4) !== 0) {
  //   balances.push(''.padStart(256, '0').split('').map(Number));
  // }

  // should include everything
  let validators = [];
  let validatorIndexes = [];
  let validatorDepositIndexes = [];
  // Should be the balances of the validators
  let balances = [];
  const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );
  let balancesIndexes = [];

  const balancesTree = new Tree(balancesView.node);

  // get validator commitment root from redis
  let validatorCommitmentRoot = [];


  // load proofs for the validators from redis
  let validatorCommitmentProofs = [];

  let balancesInput = {
    balancesRoot: ssz.capella.BeaconState.fields.balances.hashTreeRoot(
      beaconState.balances,
    ),
    balances: balances,
    balancesProofs: balancesIndexes.map(index =>
      balancesTree.getSingleProof(index),
    ),
    validatorCommitmentRoot: validatorCommitmentRoot,
    validatorDepositIndexes: validatorDepositIndexes,
    validatorIndexes: validatorIndexes,
    validatorCommitmentProofs: validatorCommitmentProofs,
    // Should mark the zero validators
    validatorIsNotZero: [],
    currentEpoch: computeEpochAt(beaconState.slot),
    currentEth1DepositIndex: beaconState.eth1DepositIndex,
  };
})();
