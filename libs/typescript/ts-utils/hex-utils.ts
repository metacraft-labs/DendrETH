import { formatHex } from './bls';

export function arrayToHex(arr: Uint8Array): string {
  return '0x' + Buffer.from(arr).toString('hex');
}

export function hexToArray(hex: string): Buffer {
  hex = hex.startsWith('0x') ? hex.slice(2) : hex;
  return Buffer.from(hex, 'hex');
}

export function hexToBits(hex: string, numbersOfBits = 256) {
  return BigInt('0x' + formatHex(hex))
    .toString(2)
    .padStart(numbersOfBits, '0')
    .split('')
    .map(Number);
}

export function bitsToHex(bits: number[]) {
  const bitsStr = bits.join('');
  const bytesStr =
    '0x' +
    bitsStr
      .match(/.{1,8}/g)!
      .map(byte =>
        BigInt('0b' + byte)
          .toString(16)
          .padStart(2, '0'),
      )
      .join('');

  return bytesStr;
}

export function hexToLittleEndianBigInt(bytes: Uint8Array) {
  const view = new DataView(bytes.buffer);
  return view.getBigUint64(0, true);
}
