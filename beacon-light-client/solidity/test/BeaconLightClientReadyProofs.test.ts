import * as path from 'path';
import { ethers } from 'hardhat';
import { sha256 } from 'ethers/lib/utils';

import { getFilesInDir, Proof } from './utils';
import { convertProofToSolidityCalldata } from '@dendreth/utils/ts-utils/zk-utils';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

import INITIAL_UPDATE from '../../../vendor/eth2-light-client-updates/mainnet/deneb-update-284/update_9265121_9273312.json';

const logger = getGenericLogger();

describe('BeaconLightClientReadyProofs', async function () {
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
      'mainnet',
      'deneb-update-284',
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
    const FORK_VERSION = '0x04000000';
    const genesis_validators_root =
      '0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95';
    const DOMAIN_SYNC_COMMITTEE = '0x07000000';

    let result = sha256(
      FORK_VERSION.padEnd(66, '0') + genesis_validators_root.slice(2),
    );
    blc = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(
      INITIAL_UPDATE.attestedHeaderRoot,
      INITIAL_UPDATE.attestedHeaderSlot,
      INITIAL_UPDATE.finalizedHeaderRoot,
      INITIAL_UPDATE.finalizedExecutionStateRoot,
      DOMAIN_SYNC_COMMITTEE + result.slice(2, 58),
    );
  });

  it('Importing real data', async function () {
    logger.info(' >>> Begin importing of real updates');
    for (let i = 1; i < updates.length; i++) {
      const proof = await convertProofToSolidityCalldata(proofs[i], publics[i]);

      logger.info(` >>> Importing update ${i}...`);

      const transaction = await blc.lightClientUpdate(
        { ...proof, ...updates[i] },
        {
          gasLimit: 30000000,
        },
      );

      const result = await transaction.wait();

      logger.info(` >>> Successfully imported update ${i}!`);
    }
  });
});
