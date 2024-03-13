// import yargs from 'yargs';
// import { Redis as RedisLocal } from '../../../relay/implementations/redis';
// import config from '../common_config.json';
// const {
//   KeyPrefix,
//   WorkQueue,
//   Item,
// } = require('@mevitae/redis-work-queue/dist/WorkQueue');
// import CONSTANTS from '../constants/validator_commitment_constants.json';
// import {
//   BeaconApi,
//   getBeaconApi,
// } from '../../../relay/implementations/beacon-api';
// import { Tree } from '@chainsafe/persistent-merkle-tree';
// import { computeEpochAt } from '../../../libs/typescript/ts-utils/ssz-utils';
// import { readFileSync } from 'fs';
// import {
//   convertValidatorToProof,
//   getCommitmentMapperProof,
//   getNthParent,
//   getZeroValidatorInput,
//   gindexFromIndex,
// } from '../validators_commitment_mapper_tree/utils';
// import {
//   panic,
//   splitIntoBatches,
// } from '../../../libs/typescript/ts-utils/common-utils';
// import {
//   BalancesAccumulatorInput,
//   Validator,
//   ValidatorPoseidonInput,
// } from '../../../relay/types/types';
// import Redis from 'ioredis';
// import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
// import {
//   convertValidatorToValidatorPoseidonInput,
//   getZeroValidatorPoseidonInput,
// } from './utils';
// import { CommandLineOptionsBuilder } from '../cmdline';
//
// const CIRCUIT_SIZE = 2;
// let TAKE: number | undefined;
//
// enum TaskTag {
//   FIRST_LEVEL = 0,
// }
//
// (async () => {
//   const { ssz } = await import('@lodestar/types');
//
//   const options = new CommandLineOptionsBuilder()
//     .withRedisOpts()
//     .usage(
//       'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
//     )
//     .option('beacon-node', {
//       alias: 'beacon-node',
//       describe: 'The beacon node url',
//       type: 'string',
//       default: config['beacon-node'],
//       description: 'Sets a custom beacon node url',
//     })
//     .option('slot', {
//       alias: 'slot',
//       describe: 'The state slot',
//       type: 'number',
//       default: undefined,
//       description: 'Fetches the balances for this slot',
//     })
//     .option('take', {
//       alias: 'take',
//       describe: 'The number of validators to take',
//       type: 'number',
//       default: undefined,
//       description: 'Sets the number of validators to take',
//     })
//     .option('offset', {
//       alias: 'offset',
//       describe: 'Index offset in the validator set',
//       type: 'number',
//       default: undefined,
//     })
//     .option('protocol', {
//       alias: 'protocol',
//       describe: 'The protocol',
//       type: 'string',
//       default: 'demo',
//       description: 'Sets the protocol',
//     })
//     .build();
//
//   const redis = new RedisLocal(options['redis-host'], options['redis-port']);
//   const db = new Redis(
//     `redis://${options['redis-host']}:${options['redis-port']}`,
//   );
//
//   TAKE = options['take'];
//
//   const first_level_queue = new WorkQueue(
//     new KeyPrefix(`${CONSTANTS.balanceVerificationAccumulatorProofQueue}:0`),
//   );
//
//   const beaconApi = await getBeaconApi(options['beacon-node']);
//   const slot =
//     options['slot'] !== undefined
//       ? options['slot']
//       : Number(await beaconApi.getHeadSlot());
//
//   const { beaconState } =
//     (await beaconApi.getBeaconState(slot)) ||
//     panic('Could not fetch beacon state');
//
//   const offset = Number(options['offset']) || 0;
//   const take = TAKE !== undefined ? TAKE + offset : undefined;
//
//   beaconState.balances = beaconState.balances.slice(offset, take);
//   beaconState.validators = beaconState.validators.slice(offset, take);
//
//   let validatorsAccumulator: any[] = JSON.parse(
//     readFileSync('validators.json', 'utf-8'),
//   );
//
//   validatorsAccumulator.map((x, i) => {
//     x.validator_index = i;
//     return x;
//   });
//
//   // Should be the balances of the validators
//   const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
//     beaconState.balances,
//   );
//
//   const balancesTree = new Tree(balancesView.node);
//
//   let balancesProofs = validatorsAccumulator.map(v => {
//     return balancesTree
//       .getSingleProof(gindexFromIndex(BigInt(v.validator_index), 39n) + 1n)
//       .map(bytesToHex)
//       .slice(0, 22);
//   });
//
//   // get validator commitment root from redis
//   let validatorCommitmentRoot = await redis.getValidatorCommitmentRoot(
//     computeEpochAt(beaconState.slot),
//   );
//
//   // load proofs for the validators from redis
//   let validatorCommitmentProofs = await Promise.all(
//     validatorsAccumulator.map(async v => {
//       return (
//         await getCommitmentMapperProof(
//           BigInt(computeEpochAt(beaconState.slot)),
//           gindexFromIndex(BigInt(v.validator_index), 40n),
//           'poseidon',
//           redis,
//         )
//       ).slice(0, 24);
//     }),
//   );
//
//   let balancesInputs: BalancesAccumulatorInput[] = [];
//   for (
//     let chunkIdx = 0;
//     chunkIdx <
//     Math.floor(
//       (validatorsAccumulator.length + CIRCUIT_SIZE - 1) / CIRCUIT_SIZE,
//     );
//     chunkIdx++
//   ) {
//     let balancesInput: BalancesAccumulatorInput = {
//       balancesRoot: bytesToHex(balancesTree.getRoot(65536n)),
//       balances: [],
//       balancesProofs: [],
//       validatorDepositIndexes: [],
//       validatorIndexes: [],
//       validatorCommitmentProofs: [],
//       validatorIsNotZero: [],
//       validators: [],
//       validatorCommitmentRoot: validatorCommitmentRoot,
//       currentEpoch: computeEpochAt(beaconState.slot),
//       currentEth1DepositIndex: beaconState.eth1DepositIndex,
//     };
//     for (let j = 0; j < CIRCUIT_SIZE; j++) {
//       const idx = chunkIdx * CIRCUIT_SIZE + j;
//       if (idx < validatorsAccumulator.length) {
//         balancesInput.balances.push(
//           bytesToHex(
//             balancesTree.getNode(gindexFromIndex(BigInt(idx), 38n) + 1n).root,
//           ),
//         );
//         balancesInput.balancesProofs.push(balancesProofs[idx]);
//         balancesInput.validatorDepositIndexes.push(
//           validatorsAccumulator[idx].validator_index,
//         );
//         balancesInput.validatorIndexes.push(
//           Number(gindexFromIndex(BigInt(idx), 22n) + 1n),
//         );
//         balancesInput.validators.push(
//           convertValidatorToValidatorPoseidonInput(beaconState.validators[idx]),
//         );
//         balancesInput.validatorCommitmentProofs.push(
//           validatorCommitmentProofs[idx],
//         );
//         balancesInput.validatorIsNotZero.push(1);
//       } else {
//         balancesInput.balances.push(''.padStart(64, '0'));
//         balancesInput.balancesProofs.push(
//           new Array(22).map(x => ''.padStart(64, '0')),
//         );
//         balancesInput.validators.push(getZeroValidatorPoseidonInput());
//         balancesInput.validatorDepositIndexes.push(0);
//         balancesInput.validatorIndexes.push(0);
//         balancesInput.validatorCommitmentProofs.push(
//           new Array(22).map(x => new Array(4).fill(0)),
//         );
//         balancesInput.validatorIsNotZero.push(0);
//       }
//     }
//
//     balancesInputs.push(balancesInput);
//   }
//
//   await redis.saveBalancesAccumulatorInput(balancesInputs, options['protocol']);
//   scheduleFirstLevelTasks(balancesInputs);
//
//   db.quit();
//   await redis.quit();
//
//   async function scheduleFirstLevelTasks(
//     balancesInputs: BalancesAccumulatorInput[],
//   ) {
//     for (let i = 0; i < balancesInputs.length; i++) {
//       const buffer = new ArrayBuffer(8);
//       const dataView = new DataView(buffer);
//       dataView.setBigUint64(0, BigInt(i), false);
//       first_level_queue.addItem(db, new Item(buffer));
//     }
//   }
// })();
