import * as fs from "fs";
import * as path from "path";
import { PointG1, PointG2 } from "@noble/bls12-381";
import {
  bigint_to_array,
  formatHex,
  hexToBytes,
  utils
} from "./bls";
import { ssz } from "@chainsafe/lodestar-types";

const hashToField = utils.hashToField;

export function getFilesInDir(_path) {
  let files = [];
  const content = fs.readdirSync(_path, { encoding: 'utf-8', withFileTypes: true });
  for (let f of content) {
    if (f.isDirectory()) {
      files = [...files, ...getFilesInDir(path.join(_path, f.name))];
    } else {
      files.push(fs.readFileSync(path.join(_path, f.name)));
    }
  }
  return files;
}


export async function getInputSignature(pubkey: string, signature: string, blockRoot: string) {
  let pubkeyPoint = PointG1.fromHex(formatHex(pubkey));
  pubkeyPoint.assertValidity();

  let signaturePoint = PointG2.fromSignature(formatHex(signature));
  signaturePoint.assertValidity();
  let root = hexToBytes(blockRoot);

  const genesisValidatorsRoot: Uint8Array = hexToBytes(
    "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"
  );

  const ForkData = ssz.phase0.ForkData;
  let fork_data = ForkData.defaultValue();
  fork_data.currentVersion = hexToBytes("0x00000000");

  fork_data.genesisValidatorsRoot = genesisValidatorsRoot;
  let fork_data_root = ForkData.hashTreeRoot(fork_data);

  let domain = new Uint8Array(32);
  const DOMAIN_BEACON_PROPOSER = Uint8Array.from([0, 0, 0, 0]);
  for (let i = 0; i < 4; i++) domain[i] = DOMAIN_BEACON_PROPOSER[i];
  for (let i = 0; i < 28; i++) domain[i + 4] = fork_data_root[i];

  const SigningData = ssz.phase0.SigningData;
  let signing_data = SigningData.defaultValue();
  signing_data.objectRoot = hexToBytes(blockRoot);
  signing_data.domain = domain;
  let signing_root: Uint8Array = SigningData.hashTreeRoot(signing_data);

  let u: bigint[][] = await hashToField(signing_root, 2);

  let pubkeyArray: string[][] = [
    bigint_to_array(55, 7, BigInt("0x" + pubkeyPoint.toAffine()[0].value.toString(16))),
    bigint_to_array(55, 7, BigInt("0x" + pubkeyPoint.toAffine()[1].value.toString(16))),
  ];

  let signatureArray: string[][][] = [
    [
      bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[0].c0.value.toString(16))),
      bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[0].c1.value.toString(16))),
    ],
    [
      bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[1].c0.value.toString(16))),
      bigint_to_array(55, 7, BigInt("0x" + signaturePoint.toAffine()[1].c1.value.toString(16))),
    ],
  ];

  let hashArray: string[][][] = [
    [
      bigint_to_array(55, 7, BigInt("0x" + u[0][0].toString(16))),
      bigint_to_array(55, 7, BigInt("0x" + u[0][1].toString(16))),
    ],
    [
      bigint_to_array(55, 7, BigInt("0x" + u[1][0].toString(16))),
      bigint_to_array(55, 7, BigInt("0x" + u[1][1].toString(16))),
    ],
  ];

  return {
    pubkey: pubkeyArray,
    signature: signatureArray,
    hash: hashArray
  }
}
