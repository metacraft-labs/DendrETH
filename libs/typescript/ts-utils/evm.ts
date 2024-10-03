const networks = ['mainnet', 'sepolia', 'chiado', 'lukso'] as const;

export type NetworkName = (typeof networks)[number];

export const isNetworkName = (value: string): value is NetworkName => {
  return networks.includes(value as NetworkName);
};

export const parseNetworkName = (value: string): NetworkName | null => {
  if (isNetworkName(value)) {
    return value as NetworkName;
  }
  return null;
};

export type EthereumAddress = string & { __brand: 'EthereumAddress' };
export const ethereumAddress = (
  hexDataString: string,
): EthereumAddress | null => {
  const isValid = /^0x([0-9a-fA-F]{40})$/.test(hexDataString);
  if (isValid) {
    return hexDataString as EthereumAddress;
  }
  return null;
};

export type Hash32Byte = string & { __brand: '32 byte hex string' };
export const hash32byte = (hexDataString: string): Hash32Byte | null => {
  const isValid = /^0x([0-9a-fA-F]{64})$/.test(hexDataString);
  if (isValid) {
    return hexDataString as Hash32Byte;
  }
  return null;
};

export type TxHash = string & { __brand: 'EVM transaction hash' };
export const txHash = (hexDataString: string): TxHash | null => {
  const hash = hash32byte(hexDataString);
  if (hash !== null) {
    return hexDataString as TxHash;
  }
  return null;
};

export const explorerUrls: Record<string, any> = {
  mainnet: {
    tx: txHash => `https://etherscan.io/tx/${txHash}`,
    address: address => `https://etherscan.io/address/${address}`,
  },
  sepolia: {
    tx: txHash => `https://sepolia.etherscan.io/tx/${txHash}`,
    address: address => `https://sepolia.etherscan.io/address/${address}`,
  },
  chiado: {
    tx: txHash => `https://gnosis-chiado.blockscout.com/tx/${txHash}`,
    address: address =>
      `https://gnosis-chiado.blockscout.com/address/${address}`,
  },
  lukso: {
    tx: txHash =>
      `https://explorer.consensus.testnet.lukso.network/tx/${txHash}`,
    address: address =>
      `https://explorer.consensus.testnet.lukso.network/address/${address}`,
  },
} satisfies {
  [network in NetworkName]?: {
    tx: (tx: TxHash) => string;
    address: (address: EthereumAddress) => string;
  };
};
