import { fromBinary } from '@cosmjs/cosmwasm-stargate';
import { readFileSync } from 'fs';
import { hexToBytes } from '@dendreth/utils/ts-utils/bls';

function toHex(number: string) {
  return BigInt(number).toString(16).padStart(64, '0');
}

const inputVkPath = process.argv[2];

const vk = JSON.parse(readFileSync(inputVkPath, 'utf8'));

const g16_alpha1 = hexToBytes(
  toHex(vk.vk_alpha_1[0]) + toHex(vk.vk_alpha_1[1]),
).toString();

const g16_beta2 = hexToBytes(
  toHex(vk.vk_beta_2[0][1]) +
    toHex(vk.vk_beta_2[0][0]) +
    toHex(vk.vk_beta_2[1][1]) +
    toHex(vk.vk_beta_2[1][0]),
).toString();

const g16_gamma2 = hexToBytes(
  toHex(vk.vk_gamma_2[0][1]) +
    toHex(vk.vk_gamma_2[0][0]) +
    toHex(vk.vk_gamma_2[1][1]) +
    toHex(vk.vk_gamma_2[1][0]),
).toString();

const g16_delta2 = hexToBytes(
  toHex(vk.vk_delta_2[0][1]) +
    toHex(vk.vk_delta_2[0][0]) +
    toHex(vk.vk_delta_2[1][1]) +
    toHex(vk.vk_delta_2[1][0]),
).toString();

let g16_snark_ic_hex = '';

for (let i = 0; i < vk.IC.length; i++) {
  g16_snark_ic_hex += toHex(vk.IC[i][0]) + toHex(vk.IC[i][1]);
}

const g16_snark_ic = hexToBytes(g16_snark_ic_hex).toString();

const result = `
const uint8_t g16_alpha1[] = {${g16_alpha1}};
const uint8_t g16_beta2[] ={${g16_beta2}};
const uint8_t g16_gamma2[] = {${g16_gamma2}};
const uint8_t g16_delta2[] =  {${g16_delta2}};

const uint8_t g16_snark_ic[] =  {${g16_snark_ic}};
`;

console.log(result);
