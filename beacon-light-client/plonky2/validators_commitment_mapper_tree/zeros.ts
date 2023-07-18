import { sha256 } from 'ethers/lib/utils';
import { bytesToHex, formatHex } from '../../../libs/typescript/ts-utils/bls';

let zeros: string[] = [];

zeros[0] = ''.padStart(64, '0');

for (let i = 1; i <= 40; i++) {
  zeros[i] = formatHex(sha256('0x' + zeros[i - 1] + zeros[i - 1]));
}
const lengthBuf = Buffer.alloc(32);
lengthBuf.writeUIntLE(0, 0, 6);

console.log(sha256('0x' + zeros[40] + bytesToHex(new Uint8Array(lengthBuf))));

console.log(zeros[40]);
