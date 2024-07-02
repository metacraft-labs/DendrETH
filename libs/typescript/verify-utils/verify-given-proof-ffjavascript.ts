import { concat } from 'ethers/lib/utils';
import { Scalar, buildBn128 } from 'ffjavascript';
import { unstringifyBigInts, bitTo2BigInts } from '@/ts-utils/common-utils';
import * as fs from 'fs';

async function getCurveFromName(name) {
  let curve;
  const normName = normalizeName(name);
  if (['BN128', 'BN254', 'ALTBN128'].indexOf(normName) >= 0) {
    curve = await buildBn128();
  } else {
    throw new Error(`Curve not supported: ${name}`);
  }
  return curve;

  function normalizeName(n) {
    return n
      .toUpperCase()
      .match(/[A-Za-z0-9]+/g)
      .join('');
  }
}

async function groth16Verify(
  vk_verifier,
  proof,
  currentHeaderHash,
  attestedHeaderRoot,
  finalizedHeaderRoot,
  finalizedExecutionStateRoot,
  slot,
  domain,
) {
  var zerosBytes: number[] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0,
  ];

  for (let i = 0; i < 8 - slot.toString().length; i++) {
    zerosBytes = zerosBytes.concat(0);
  }

  const hash = concat([
    currentHeaderHash,
    attestedHeaderRoot,
    finalizedHeaderRoot,
    finalizedExecutionStateRoot,
    zerosBytes,
    slot,
    domain,
  ]);

  const publicSignals = bitTo2BigInts(hash);

  const curve = await getCurveFromName('bn128');

  const IC0 = curve.G1.fromObject(vk_verifier.IC[0]);
  const IC = new Uint8Array(curve.G1.F.n8 * 2 * publicSignals.length);
  const w = new Uint8Array(curve.Fr.n8 * publicSignals.length);

  for (let i = 0; i < publicSignals.length; i++) {
    const buffP = curve.G1.fromObject(vk_verifier.IC[i + 1]);
    IC.set(buffP, i * curve.G1.F.n8 * 2);
    Scalar.toRprLE(w, curve.Fr.n8 * i, publicSignals[i], curve.Fr.n8);
  }

  let cpub = await curve.G1.multiExpAffine(IC, w);
  cpub = curve.G1.add(cpub, IC0);

  const pi_a = curve.G1.fromObject(proof.pi_a);
  const pi_b = curve.G2.fromObject(proof.pi_b);
  const pi_c = curve.G1.fromObject(proof.pi_c);

  const vk_gamma_2 = curve.G2.fromObject(vk_verifier.vk_gamma_2);
  const vk_delta_2 = curve.G2.fromObject(vk_verifier.vk_delta_2);
  const vk_alpha_1 = curve.G1.fromObject(vk_verifier.vk_alpha_1);
  const vk_beta_2 = curve.G2.fromObject(vk_verifier.vk_beta_2);

  const res = await curve.pairingEq(
    curve.G1.neg(pi_a),
    pi_b,
    cpub,
    vk_gamma_2,
    pi_c,
    vk_delta_2,

    vk_alpha_1,
    vk_beta_2,
  );
  curve.terminate();

  return res;
}

export async function VerifyFromPaths(
  keyPath,
  proofPath,
  updatePathOld,
  updatePath,
) {
  const vk_verifierSTR = fs.readFileSync(keyPath, 'utf-8');
  const vk_verifier = unstringifyBigInts(JSON.parse(vk_verifierSTR));

  const proofSTR = fs.readFileSync(proofPath, 'utf-8');
  const proof = unstringifyBigInts(JSON.parse(proofSTR));

  const updateOldSTR = fs.readFileSync(updatePathOld, 'utf-8');
  const updateOld = JSON.parse(updateOldSTR);

  const updateSTR = fs.readFileSync(updatePath, 'utf-8');
  const update = JSON.parse(updateSTR);

  const currentHeaderHash = updateOld.attestedHeaderRoot;
  const attestedHeaderRoot = update.attestedHeaderRoot;
  const slot = update.attestedHeaderSlot;
  const finalizedHeaderRoot = update.finalizedHeaderRoot;
  const finalizedExecutionStateRoot = update.finalizedExecutionStateRoot;
  const domain =
    '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';

  const res = await groth16Verify(
    vk_verifier,
    proof,
    currentHeaderHash,
    attestedHeaderRoot,
    finalizedHeaderRoot,
    finalizedExecutionStateRoot,
    slot,
    domain,
  );

  return res;
}
