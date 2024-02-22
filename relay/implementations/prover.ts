import { IProver } from '../abstraction/prover-interface';
import { ProofInputType, Proof, WitnessGeneratorInput } from '../types/types';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();
export class Prover implements IProver {
  private proverServerURL: string;

  constructor(proverServerURL: string) {
    this.proverServerURL = proverServerURL;
  }

  async genProof(proofInput: ProofInputType, mock = false): Promise<Proof> {
    var proof;
    var publicVars;
    if (!mock) {
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

      proof = JSON.parse(st.proof);

      publicVars = JSON.parse(st.pubData);
    } else {
      proof = {
        pi_a: [
          '12763925006581137919304949334797603802458106302550743441499138660117674458843',
          '3210277313367297960978284600070647965688928976864111405932126995759004567274',
          '1',
        ],
        pi_b: [
          [
            '13262650684780645924631597816699461845882952491020847346256482683974830302382',
            '18382552761441724027130145449451577465099488205434023208725500890682382654249',
          ],
          [
            '5742524743116358027698900120745802615824383991987775533210879169548095928215',
            '9031854286059022594570625510067083999168656248388663258862090924521740417974',
          ],
          ['1', '0'],
        ],
        pi_c: [
          '13764977866279122478221420133386876215994100264892193449636036345427191382230',
          '16905083775607366928536120749218882272365033612066779646866782727491849960912',
          '1',
        ],
      };
      publicVars = [
        '9734125334797249825003723072622467244897033234414344070397312864656672300128',
        '4',
      ];
    }

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
