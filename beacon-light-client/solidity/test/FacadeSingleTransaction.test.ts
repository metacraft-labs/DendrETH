import * as path from 'path';
import { ethers } from 'hardhat';
import { getFilesInDir, Proof } from './utils';
import { convertProofToSolidityCalldata } from '../../../libs/typescript/ts-utils/zk-utils';
import INITIAL_UPDATE from '../../../vendor/eth2-light-client-updates/prater/capella-updates/update_5200024_5200056.json';

describe.only('BeaconLightClientReadyProofs', async function () {
  let proofs: Proof[];
  let publics: any[];
  let updates: any[];

  let blc;
  let facadeSingleTransaction;

  beforeEach(async function () {
    const dir = path.join(
      __dirname,
      '..',
      '..',
      '..',
      'vendor',
      'eth2-light-client-updates',
      'prater',
      'capella-updates',
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
      INITIAL_UPDATE.attested_header_root,
      INITIAL_UPDATE.finalized_header_root,
      INITIAL_UPDATE.finalized_execution_state_root,
    );

    facadeSingleTransaction = await (
      await ethers.getContractFactory('contracts/hashi/FacadeSingleTransaction.sol:OracleAdapterFacade')
    ).deploy(blc.address);
  });

  it('Importing real data', async function () {
    console.log(' >>> Begin importing of real updates');
    for (let i = 1; i < updates.length; i++) {
      const proof = await convertProofToSolidityCalldata(proofs[i], publics[i]);

      console.log(` >>> Importing update ${i}...`);

      const transaction = await facadeSingleTransaction.updateHash(
        0,
        i,
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
