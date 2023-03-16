import { exec as _exec } from 'child_process';
import { readFile, rm, writeFile } from 'fs/promises';
import path from 'path';
import { promisify } from 'util';
import { IProver } from '../abstraction/prover-interface';
import { ProofInputType, Proof } from '../types/types';
const exec = promisify(_exec);

export class Prover implements IProver {
  private witnessGeneratorPath: string;
  private rapidSnarkProverPath: string;
  private zkeyFilePath: string;

  constructor(
    witnessGeneratorPath: string,
    rapidSnarkProverPath: string,
    zkeyFilePath: string,
  ) {
    this.witnessGeneratorPath = witnessGeneratorPath;
    this.rapidSnarkProverPath = rapidSnarkProverPath;
    this.zkeyFilePath = zkeyFilePath;
  }

  async genProof(proofInput: ProofInputType): Promise<Proof> {
    await writeFile(
      path.join(
        __dirname,
        `input_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
      ),
      JSON.stringify(proofInput.proofInput),
    );

    await exec(
      `${this.witnessGeneratorPath} ${path.join(
        __dirname,
        `input_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
      )} ${path.join(
        __dirname,
        `witness_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.wtns`,
      )}`,
    );

    await exec(
      `${this.rapidSnarkProverPath} ${this.zkeyFilePath} ${path.join(
        __dirname,
        `witness_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.wtns`,
      )} ${path.join(
        __dirname,
        `proof_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
      )} ${path.join(
        __dirname,
        `public_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
      )}`,
    );

    const proof = JSON.parse(
      await readFile(
        path.join(
          __dirname,
          `proof_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
        ),
        'utf-8',
      ),
    );

    const publicVars = JSON.parse(
      await readFile(
        path.join(
          __dirname,
          `public_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
        ),
        'utf-8',
      ),
    );

    // remove files
    Promise.all([
      rm(
        path.join(
          __dirname,
          `witness_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.wtns`,
        ),
      ),
      rm(
        path.join(
          __dirname,
          `input_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
        ),
      ),
      rm(
        path.join(
          __dirname,
          `proof_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
        ),
      ),
      rm(
        path.join(
          __dirname,
          `public_${proofInput.prevUpdateSlot}_${proofInput.updateSlot}.json`,
        ),
      ),
    ]);

    return {
      pi_a: proof.pi_a,
      pi_b: proof.pi_b,
      pi_c: proof.pi_c,
      public: [...publicVars],
    };
  }
}
