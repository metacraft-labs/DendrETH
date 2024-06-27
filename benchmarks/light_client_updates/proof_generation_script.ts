import { ProofInputType } from '@dendreth/relay/types/types';
import { Prover } from '@dendreth/relay/implementations/prover';
import { IProver } from '@dendreth/relay/abstraction/prover-interface';
import fs from 'fs';
import path from 'path';

// Function to process each proof file
const processProofFile = async (filePath: string, prover: IProver) => {
  const jsonData = fs.readFileSync(filePath, 'utf-8');
  const proofInput: ProofInputType = JSON.parse(jsonData);

  const proof = await prover.genProof(proofInput);

  const result = {
    ...proofInput,
    proof,
  };

  const outputFilePath = filePath.replace('proof_', 'result_');
  fs.writeFileSync(outputFilePath, JSON.stringify(result, null, 2));
  console.log(`Proof generated and saved to ${outputFilePath}`);
};

// Main function to process all proof files
(async () => {
  const proverServerUrl = 'http://localhost:5000'; // Replace with your prover server URL
  const prover: Prover = new Prover(proverServerUrl); // Initialize your prover here
  const buildDir = path.join(__dirname, 'build');
  const proofFiles = fs
    .readdirSync(buildDir)
    .filter(file => file.startsWith('proof_') && file.endsWith('.json'));

  for (const file of proofFiles) {
    const filePath = path.join(buildDir, file);
    try {
      await processProofFile(filePath, prover);
    } catch (error) {
      console.error(`Error processing ${filePath}:`, error);
    }
  }
})();
