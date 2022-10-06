import * as fs from 'fs';
import * as path from 'path';
import hre, { ethers } from 'hardhat';
import { getConstructorArgs } from '../../../../tasks/utils';
import { CONTRACTS_TEMP_PATH, GAS_REPORTABLE, REPORT_PATH } from './constants';
import {
  writeContract,
  clearTempDir,
  getContracts,
  rewriteImports,
} from './crud';
import { stringify, numerate } from './format';
import { getContractFunctions } from './function';
import { setupGasReportable } from './log';
import { Function } from './types';
import {
  FormatedJsonUpdate,
  formatJSONUpdate,
  formatLightClientUpdate,
} from '../../format';
import { getFilesInDir, getSolidityProof } from '../..';
import * as constants from '../../constants';
import {
  changeContractsNames,
  getContractsNames,
  markKeyLines,
  stringifyReport,
} from './helpers';

(async () => {
  try {
    clearTempDir();

    const report = {};
    const network = 'mainnet';
    const updates = getFilesInDir(
      path.join(
        __dirname,
        '..',
        '..',
        '..',
        '..',
        '..',
        '..',
        'vendor',
        'eth2-light-client-updates',
        network,
        'updates',
      ),
    )
      .slice(0, 2)
      .map(u =>
        formatJSONUpdate(
          JSON.parse(u.toString()),
          constants.GENESIS_FORK_VERSION.join(''),
        ),
      );

    const contracts0 = getContracts();
    const contracts1 = getContractsNames(contracts0);
    const contracts2 = changeContractsNames(contracts0, contracts1);
    const contracts3 = contracts2
      .map(c => setupGasReportable(rewriteImports(c)))
      .map(markKeyLines);

    const fids = {};
    for (let i = 0; i < contracts3.length; i++) {
      const c0 = contracts0[i];
      const f0 = getContractFunctions(numerate(c0));
      const c3 = contracts3[i];
      const f3 = getContractFunctions(numerate(c3));
      for (let j = 0; j < f0.length; j++) {
        const o0 = f0[j];
        const o3 = f3[j];
        o3.modifiers[0].includes('gas_report')
          ? (fids[
              parseInt(
                o3.modifiers[0]
                  .replace('gas_report', '')
                  .trim()
                  .replace('(', '')
                  .trim()
                  .replace(')', '')
                  .trim(),
              )
            ] = o0)
          : null;
      }
    }

    fs.mkdirSync(CONTRACTS_TEMP_PATH);
    writeContract(
      GAS_REPORTABLE.toString(),
      path.join(CONTRACTS_TEMP_PATH, 'GasReportable'.concat('.sol')),
    );
    contracts3.map((c, i) => {
      writeContract(
        stringify(c).replaceAll(' pure', '  ').replaceAll(' view', '  '),
        path.join(CONTRACTS_TEMP_PATH, contracts1[i].newn.concat('.sol')),
      );
    });
    await hre.run('compile', { force: true });
    clearTempDir();
    const instance = await (
      await ethers.getContractFactory('BeaconLightClientGasReportable')
    ).deploy(...getConstructorArgs(network), { gasLimit: 30000000 });

    const update = async (u1: FormatedJsonUpdate, u2: FormatedJsonUpdate) => {
      const p = await getSolidityProof(u1, u2, network);
      const u = formatLightClientUpdate(u2, p);
      const t = await instance.light_client_update(u);
      const r = await t.wait();
      r.events
        ?.filter(({ event }) => event === 'LogStart')
        .map(({ args }) => {
          report[fids[args.y.toNumber()].signature] = {};
          report[fids[args.y.toNumber()].signature].gs = args.z.toNumber();
        });
      r.events
        ?.filter(({ event }) => event === 'LogEnd')
        .map(({ args }) => {
          report[fids[args.y.toNumber()].signature].ge = args.z.toNumber();
        });
      r.events
        ?.filter(({ event }) => event === 'LogLine')
        .map(({ args }) => {
          const id = args.y.toNumber();
          let l = (fids[id] as Function).body.find(
            l => l.index === args.x.toNumber(),
          );
          if (l !== undefined && l.content.trim().length === 0) return;
          if (l === undefined) {
            l = (fids[id] as Function).declaration.find(
              l => l.index === args.x.toNumber(),
            );
            if (l === undefined) return;
          }

          const _id = fids[id].signature;
          if (report[_id] === undefined) return;

          if (report[_id].kli === undefined) {
            report[_id].kli = new Set();
            report[_id].klgl = {};
          }

          report[_id].kli.add(l.index);
          report[_id].klgl[l.index] = args.z.toNumber();
        });

      for (let id of Object.keys(report)) {
        report[id].gst = report[id].gs - report[id].ge;
        delete report[id].gs;
        delete report[id].ge;
      }

      for (let fSig of Object.keys(report) as any) {
        const { kli, klgl } = report[fSig];
        if (kli === undefined) continue;

        report[fSig].klgt = {};
        const v = (kli as Set<number>).values();
        let f: number = -1;
        for (let keyLineIdx of v) {
          const l = Object.keys(report[fSig].klgt).length;
          if (l === 0) {
            report[fSig].klgt[keyLineIdx] = klgl[keyLineIdx];
            f = keyLineIdx;
          } else {
            const previous = Object.keys(report[fSig].klgt)[
              Object.keys(report[fSig].klgt).length - 1
            ];
            report[fSig].klgt[keyLineIdx] = klgl[previous] - klgl[keyLineIdx];
          }
        }

        delete report[fSig].kli;
        delete report[fSig].klgl;
        report[fSig].klgt[f] = 0;
      }
    };

    await update(updates[0], updates[1]);
    fs.writeFileSync(REPORT_PATH, stringifyReport(report, fids));
    const msg = ` >>> Generated a gas consumption report for the Beacon Light Client. Check out here: ${REPORT_PATH}`;
    console.log(
      Array(msg.length + 1)
        .fill('=')
        .join(''),
    );
    console.log(msg);
    console.log(
      Array(msg.length + 1)
        .fill('=')
        .join(''),
    );
  } catch (err) {
    console.error(err);
    clearTempDir();
  }
})();
