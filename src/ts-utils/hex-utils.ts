export function arrayToHex(arr: Uint8Array): string {
    return '0x' + Buffer.from(arr).toString('hex');
  }

  export function hexToArray(hex: string): Buffer {
    hex = hex.startsWith('0x') ? hex.slice(2) : hex;
    return Buffer.from(hex, 'hex');
  }