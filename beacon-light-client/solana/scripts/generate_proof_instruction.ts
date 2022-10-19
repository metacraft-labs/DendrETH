import * as BufferLayout from '@solana/buffer-layout';
import { Buffer } from 'buffer';

export function bnToBuf(bn: string): Uint8Array {
  var hex = BigInt(bn).toString(16).padStart(64, '0');

  var u8 = new Uint8Array(32);

  var i = 0;
  var j = 0;
  while (i < 32) {
    u8[i] = parseInt(hex.slice(j, j + 2), 16);
    i += 1;
    j += 2;
  }

  return u8;
}

export function copyTo(data: Uint8Array, source: Uint8Array, start: number) {
  for (let i = 0; i < 32; i++) {
    data[start + i] = source[31 - i];
  }
}

export function getProofInstruction(proof, public_data): Uint8Array {
  const data = new Uint8Array(384);

  const a1 = bnToBuf(proof.pi_a[0]);
  copyTo(data, a1, 0);

  const a2 = bnToBuf(proof.pi_a[1]);
  copyTo(data, a2, 32);

  const b11 = bnToBuf(proof.pi_b[0][0]);
  copyTo(data, b11, 64);
  const b12 = bnToBuf(proof.pi_b[0][1]);
  copyTo(data, b12, 96);

  const b21 = bnToBuf(proof.pi_b[1][0]);
  copyTo(data, b21, 128);

  const b22 = bnToBuf(proof.pi_b[1][1]);
  copyTo(data, b22, 160);

  const c1 = bnToBuf(proof.pi_c[0]);
  copyTo(data, c1, 192);

  const c2 = bnToBuf(proof.pi_c[1]);
  copyTo(data, c2, 224);

  const pub1 = bnToBuf(public_data[0]);
  copyTo(data, pub1, 256);

  const pub2 = bnToBuf(public_data[1]);
  copyTo(data, pub2, 288);

  const pub3 = bnToBuf(public_data[2]);
  copyTo(data, pub3, 320);

  const pub4 = bnToBuf(public_data[3]);
  copyTo(data, pub4, 352);

  return data;
}
