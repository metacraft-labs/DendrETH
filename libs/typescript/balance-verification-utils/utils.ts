import { assert } from 'console';

type Network = 'mainnet' | 'goerli' | 'sepolia';
type Address = string;
type Timestamp = string;
type AddressConfig = { [key in Network]: Address };
type TimestampConfig = { [key in Network]: Timestamp };

// LIDO withdrawal credentials are taken from https://github.com/lidofinance/docs/blob/main/docs/deployed-contracts/index.md
// The staking router contract
const lidoWithdrawalCredentials: AddressConfig = {
  mainnet: '0x010000000000000000000000b9d7934878b5fb9610b3fe8a5e441e8fad7e293f',
  goerli: '0x010000000000000000000000dc62f9e8C34be08501Cdef4EBDE0a280f576D762',
  sepolia: '0x010000000000000000000000De7318Afa67eaD6d6bbC8224dfCe5ed6e4b86d76',
};

export function getLidoWithdrawCredentials(network: Network): Address {
  return lidoWithdrawalCredentials[network];
}

const genesisBlockTimestamp: TimestampConfig = {
  mainnet: '1606824023',
  goerli: '1616508000',
  sepolia: '1655733600',
};

export function getGenesisBlockTimestamp(network: Network): Timestamp {
  return genesisBlockTimestamp[network];
}

export function isNetwork(network: string): network is Network {
  return ['mainnet', 'goerli', 'sepolia'].includes(network);
}

export function assertSupportedNetwork(x: string): Network {
  if (!isNetwork(x)) {
    throw new Error(`Unsupported network: ${x}`);
  }

  return x;
}
