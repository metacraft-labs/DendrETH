import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';
import { getBlockHeaderFromUpdate } from '../../../../libs/typescript/ts-utils/ssz-utils';
import * as UPDATE0 from '../../../circom/scripts/light_client/relayer_updates/update_237215.json';

export const getConstructorArgs = async (network: string) => {
  network = network === 'hardhat' ? 'mainnet' : network;
  const { ssz } = await import('@lodestar/types');

  console.log(
    'instantiate optimistic',
    bytesToHex(
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(
        await getBlockHeaderFromUpdate(UPDATE0.data.attested_header.beacon),
      ),
    ),
  );

  return [
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(
      await getBlockHeaderFromUpdate(UPDATE0.data.attested_header.beacon),
    ),
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(
      await getBlockHeaderFromUpdate(UPDATE0.data.finalized_header.beacon),
    ),
    UPDATE0.data.finalized_header.execution.state_root,
  ];
};
