import { groth16 } from 'snarkjs';

export async function convertProofToSolidityCalldata(proof, publicVars) {
  const calldata = await groth16.exportSolidityCallData(proof, publicVars);

  const argv: string[] = calldata
    .replace(/["[\]\s]/g, '')
    .split(',')
    .map(x => BigInt(x).toString());

  const a = [argv[0], argv[1]];
  const b = [
    [argv[2], argv[3]],
    [argv[4], argv[5]],
  ];
  const c = [argv[6], argv[7]];

  return { a, b, c };
}
