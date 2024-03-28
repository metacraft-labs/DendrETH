import { IProver } from '../abstraction/prover-interface';
import { ProofInputType, Proof, WitnessGeneratorInput } from '../types/types';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();
export class Prover implements IProver {
  private proverServerURL: string;

  constructor(proverServerURL: string) {
    this.proverServerURL = proverServerURL;
  }

  async genProof(proofInput: ProofInputType): Promise<Proof> {
    logger.info('Starting to generate proofs');

    let st = await this.getStatus();

    if (st.status == 'busy') {
      throw new Error('Proving server is not ready');
    }

    logger.info('Server is ready sending input');

    await this.callInput(proofInput.proofInput);

    logger.info('Input send waiting for proof generation');

    st = await this.getStatus();

    while (st.status == 'busy') {
      st = await this.getStatus();

      // to not overload server with requests
      await new Promise(r => setTimeout(r, 2000));
    }

    logger.info('Proof successfully generated');

    const proof = JSON.parse(st.proof);

    const publicVars = JSON.parse(st.pubData);

    return {
      pi_a: proof.pi_a,
      pi_b: proof.pi_b,
      pi_c: proof.pi_c,
      public: [...publicVars],
    };
  }

  async callInput(input: WitnessGeneratorInput) {
    const rawResponse = await fetch(
      `${this.proverServerURL}/input/light_client`,
      {
        method: 'POST',
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(input),
      },
    );

    if (rawResponse.ok) {
      return true;
    } else {
      throw new Error(rawResponse.status.toString());
    }
  }

  async getStatus() {
    const rawResponse = await fetch(`${this.proverServerURL}/status`, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
      },
    });
    if (!rawResponse.ok) {
      throw new Error(rawResponse.status.toString());
    }
    return rawResponse.json();
  }
}
