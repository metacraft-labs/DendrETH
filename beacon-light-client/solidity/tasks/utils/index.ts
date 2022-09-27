import { hashTreeRootBeaconBlock } from '../../test/utils/format';

export const getConstructorArgs = (network: string) => {
  network = network === 'hardhat' ? 'mainnet' : network;
  const UPDATE0 = require(`../../../../vendor/eth2-light-client-updates/${network}/updates/00290.json`);

  return [
    parseInt(UPDATE0.attested_header.slot),
    parseInt(UPDATE0.attested_header.proposer_index),
    UPDATE0.attested_header.parent_root,
    UPDATE0.attested_header.body_root,
    UPDATE0.attested_header.state_root,
    hashTreeRootBeaconBlock(UPDATE0.attested_header),
  ];
};
