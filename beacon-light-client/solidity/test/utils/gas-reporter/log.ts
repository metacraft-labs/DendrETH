import { ArrayifiedContract, RawContract } from './types';
import { arrayify } from './format';

const _importHardhatConsoleInFile = (a: ArrayifiedContract) => {
  for (let i = 0; i < a.length; i++) {
    if (a[i].length === 0) {
      a[i] = 'import "hardhat/console.sol";';
      break;
    }
  }
  return a;
};

export const importHardhatConsoles = (
  contracts: RawContract[],
): ArrayifiedContract[] => {
  return contracts.map(arrayify).map(_importHardhatConsoleInFile);
};
