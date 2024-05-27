import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';

(async () => {
  let beaconApi = await getBeaconApi([
    'http://unstable.sepolia.beacon-api.nimbus.team/',
  ]);

  let { beaconState, stateTree } = await beaconApi.getBeaconState(5088595n);

  const { ssz } = await import('@lodestar/types');
  const eth1_deposit_index = ssz.deneb.BeaconState.getPathInfo([
    'eth1DepositIndex',
  ]).gindex;

  beaconState.latestExecutionPayloadHeader.blockNumber;

  const executionBlockNumberIndex = ssz.deneb.BeaconState.getPathInfo([
    'latestExecutionPayloadHeader',
    'blockNumber',
  ]).gindex;

  console.log('blockNumber gindex:', executionBlockNumberIndex);

  console.log(
    'blockNumber proof length:',
    stateTree.getSingleProof(executionBlockNumberIndex).length,
  );

  console.log(
    'eth1_deposit_index proof length',
    stateTree.getSingleProof(eth1_deposit_index).length,
  );
  console.log('eth1_deposit index gindex', eth1_deposit_index);
})();
