// import { CommitmentMapperScheduler } from '../../../validators_commitment_mapper/lib/scheduler';
// import {
//   GetBalancesInputParameterType,
//   getBalancesInput,
// } from '../lib/scheduler';
// import config from '../../../../common_config.json';
//
// (async () => {
//   const options: GetBalancesInputParameterType & Record<string, any> = {
//     'beacon-node': config['beacon-node'],
//     'redis-host': config['redis-host'],
//     'redis-port': config['redis-port'],
//     slot: 9111936,
//     'sync-slot': 9111936,
//     protocol: 'demo',
//     take: 19,
//     offset: 0,
//     withdrawalCredentials:
//       '0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50',
//   };
//
//   const scheduler = new CommitmentMapperScheduler();
//   await scheduler.init(options);
//   await scheduler.start(true);
//   await scheduler.dispose();
//
//   await getBalancesInput(options);
// })();
