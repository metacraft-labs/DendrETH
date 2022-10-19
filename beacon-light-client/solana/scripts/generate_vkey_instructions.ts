import { bnToBuf, copyTo } from './generate_proof_instruction';
import * as vkey from '../../circom/build/light_client/vkey.json';


function getVkeyInstructions() {
  const data = new Uint8Array(768);
  copyTo(data, bnToBuf(vkey.vk_alpha_1[0]), 0);
  copyTo(data, bnToBuf(vkey.vk_alpha_1[1]), 32);
  copyTo(data, bnToBuf(vkey.vk_beta_2[0][0]), 64);
  copyTo(data, bnToBuf(vkey.vk_beta_2[0][1]), 96);
  copyTo(data, bnToBuf(vkey.vk_beta_2[1][0]), 128);
  copyTo(data, bnToBuf(vkey.vk_beta_2[1][1]), 160);

  copyTo(data, bnToBuf(vkey.vk_gamma_2[0][0]), 192);
  copyTo(data, bnToBuf(vkey.vk_gamma_2[0][1]), 224);
  copyTo(data, bnToBuf(vkey.vk_gamma_2[1][0]), 256);
  copyTo(data, bnToBuf(vkey.vk_gamma_2[1][1]), 288);


  copyTo(data, bnToBuf(vkey.vk_delta_2[0][0]), 320);
  copyTo(data, bnToBuf(vkey.vk_delta_2[0][1]), 352);
  copyTo(data, bnToBuf(vkey.vk_delta_2[1][0]), 384);
  copyTo(data, bnToBuf(vkey.vk_delta_2[1][1]), 416);

  copyTo(data, bnToBuf(vkey.IC[0][0]), 448);
  copyTo(data, bnToBuf(vkey.IC[0][1]), 480);
  copyTo(data, bnToBuf(vkey.IC[1][0]), 512);
  copyTo(data, bnToBuf(vkey.IC[1][1]), 544);
  copyTo(data, bnToBuf(vkey.IC[2][0]), 576);
  copyTo(data, bnToBuf(vkey.IC[2][1]), 608);
  copyTo(data, bnToBuf(vkey.IC[3][0]), 640);
  copyTo(data, bnToBuf(vkey.IC[3][1]), 672);
  copyTo(data, bnToBuf(vkey.IC[4][0]), 704);
  copyTo(data, bnToBuf(vkey.IC[4][1]), 736);

  console.log(`[${data.join(',')}]`)
}

getVkeyInstructions();
