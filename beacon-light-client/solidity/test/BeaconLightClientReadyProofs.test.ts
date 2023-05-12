import * as path from 'path';
import { ethers } from 'hardhat';
import { getFilesInDir, Proof } from './utils';
import { convertProofToSolidityCalldata } from '../../../libs/typescript/ts-utils/zk-utils';
import INITIAL_UPDATE from '../../../vendor/eth2-light-client-updates/prater/capella-updates-94/update_5601823_5609044.json';

describe.only('BeaconLightClientReadyProofs', async function () {
  let proofs: Proof[];
  let publics: any[];
  let updates: any[];

  let blc;

  beforeEach(async function () {
    const dir = path.join(
      __dirname,
      '..',
      '..',
      '..',
      'vendor',
      'eth2-light-client-updates',
      'prater',
      'capella-updates-94',
    );

    proofs = getFilesInDir(dir, 'proof*.json').map(p =>
      JSON.parse(p.toString()),
    );

    publics = getFilesInDir(dir, 'public*.json').map(p =>
      JSON.parse(p.toString()),
    );

    updates = getFilesInDir(dir, 'update*.json').map(u =>
      JSON.parse(u.toString()),
    );
  });

  beforeEach(async function () {
    blc = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(
      INITIAL_UPDATE.attestedHeaderRoot,
      INITIAL_UPDATE.attestedHeaderSlot,
      INITIAL_UPDATE.finalizedHeaderRoot,
      INITIAL_UPDATE.finalizedExecutionStateRoot,
      '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695',
    );
  });

  it('Importing real data', async function () {
    console.log(' >>> Begin importing of real updates');
    for (let i = 1; i < updates.length; i++) {
      const proof = await convertProofToSolidityCalldata(proofs[i], publics[i]);

      console.log(` >>> Importing update ${i}...`);

      const transaction = await blc.light_client_update(
        { ...proof, ...updates[i] },
        {
          gasLimit: 30000000,
        },
      );

      const result = await transaction.wait();

      console.log(` >>> Successfully imported update ${i}!`);
    }
  });
});
