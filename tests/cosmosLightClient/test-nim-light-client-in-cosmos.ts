import { beforeAll, describe, expect, test } from '@jest/globals';

import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { SSZSpecTypes } from '@dendreth/utils/ts-utils/sszSpecTypes';
import { jsonToSerializedBase64 } from '@dendreth/utils/ts-utils/ssz-utils';
import { compileNimFileToWasm } from '@dendreth/utils/ts-utils/compile-nim-to-wasm';
import {
  byteArrayToNumber,
  appendJsonFile,
  getRootDir,
} from '@dendreth/utils/ts-utils/common-utils';
import {
  setUpCosmosTestnet,
  stopCosmosNode,
} from '@dendreth/utils/cosmos-utils/testnet-setup';
import { gasUsed } from '../helpers/helpers';

const exec = promisify(exec_);

let rootDir;

describe('Light Client In Cosmos', () => {
  let gasArrayLightClient: gasUsed[] = [];
  let client: SigningCosmWasmClient;
  let _contractAddress;
  let DendrETHWalletInfo;

  const controller = new AbortController();
  const { signal } = controller;

  const mnemonic =
    'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone';
  const gasPrice = GasPrice.fromString('0.0000025ustake');
  const gasUsageFile = 'tests/cosmosLightClient/gasLightClient.json';

  beforeAll(async () => {
    rootDir = await getRootDir();

    let contractDirLightClient = rootDir + `/contracts/cosmos/light-client`;
    let nimFilePathLightClient =
      contractDirLightClient + `/lib/nim/light_client_cosmos_wrapper.nim`;
    await compileNimFileToWasm(
      nimFilePathLightClient,
      `--nimcache:"${contractDirLightClient}"/nimbuild --d:lightClientCosmos \
      -o:"${contractDirLightClient}"/nimbuild/light_client.wasm`,
    );
    let compileContractCommandLightClient = `docker run -t --rm -v "${contractDirLightClient}":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.11 .`;
    console.info(`➤ ${compileContractCommandLightClient}`);

    await exec(compileContractCommandLightClient);

    let cosmos = await setUpCosmosTestnet(mnemonic, 'light-client');
    client = cosmos.client;
    DendrETHWalletInfo = cosmos.walletInfo;
  }, 720000 /* timeout in milliseconds */);

  test('Check "LightClientStore" after initialization', async () => {
    const expectedHeaderSlot = 2375680;

    // The contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/light-client/artifacts/light_client.wasm`,
    );
    // Upload the contract
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      uploadFee,
      'Upload Cosmos Light Client contract',
    );
    console.info('Upload succeeded. Receipt:', uploadReceipt);
    // Gas used
    let uploadGas = new gasUsed('Upload Light Client', uploadReceipt.gasUsed);
    gasArrayLightClient.push(uploadGas);

    // Instantiate the contract
    const instantiateFee = calculateFee(12_500_000, gasPrice);
    const bootstrapData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientBootstrap,
      rootDir + `/vendor/eth2-light-client-updates/mainnet/bootstrap.json`,
    );

    // This contract specific message is passed to the contract
    const msg = {
      bootstrap_data: bootstrapData,
    };
    var instantiation = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      msg,
      'My instance',
      instantiateFee,
      { memo: 'Create a Cosmos Light Client instance.' },
    );

    // Gas Used
    console.info(
      `Instantiation Light Client used ` + instantiation.gasUsed + ` gas`,
    );
    let initGas = new gasUsed('Init Light Client', instantiation.gasUsed);
    gasArrayLightClient.push(initGas);

    console.info('Contract instantiated at: ', instantiation.contractAddress);
    _contractAddress = instantiation.contractAddress;

    // Query contract after initialization
    const queryResultAfterInitialization = await client.queryContractSmart(
      _contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterInitialization = byteArrayToNumber(
      queryResultAfterInitialization.slice(0, 8),
    );

    expect(headerSlotAfterInitialization).toEqual(expectedHeaderSlot);
  }, 720000);

  test('Check "LightClientStore" after one update', async () => {
    const expectedHeaderSlot = 2381376;

    const updateData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientUpdate,
      rootDir + `/vendor/eth2-light-client-updates/mainnet/updates/00290.json`,
    );
    // This contract specific message is passed to the contract
    const execMsg = {
      update: {
        update_data: updateData,
      },
    };
    // Execute contract
    const executeFee = calculateFee(12_500_000, gasPrice);
    const result = await client.execute(
      DendrETHWalletInfo.address,
      _contractAddress,
      execMsg,
      executeFee,
    );

    // Gas Used
    console.info(`Update 1 Light Client used ` + result.gasUsed + ` gas`);
    let updateGas = new gasUsed('Update 1', result.gasUsed);
    gasArrayLightClient.push(updateGas);

    const wasmEvent = result.logs[0].events.find(e => e.type === 'wasm');
    console.info(
      'The "wasm" event emitted by the contract execution:',
      wasmEvent,
    );
    // Query contract after execution ( after one update )
    const queryResultAfterOneUpdate = await client.queryContractSmart(
      _contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterOneUpdate = byteArrayToNumber(
      queryResultAfterOneUpdate.slice(0, 8),
    );
    expect(headerSlotAfterOneUpdate).toEqual(expectedHeaderSlot);
  }, 720000);

  test('Check "LightClientStore" after first 20 updates', async () => {
    const expectedHeaderSlot = 2531776;

    const updateFiles = glob(
      rootDir + `/vendor/eth2-light-client-updates/mainnet/updates/*.json`,
    );
    var counter = 1;
    for (var updateFile of updateFiles.slice(0, 20)) {
      const updateData = await jsonToSerializedBase64(
        SSZSpecTypes.LightClientUpdate,
        updateFile,
      );
      const execMsg = {
        update: {
          update_data: updateData,
        },
      };
      // Execute contract
      const executeFee = calculateFee(12_500_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        execMsg,
        executeFee,
      );
      // Gas Used
      counter++;
      console.info(
        `Update ` + counter + ` Light Client used ` + result.gasUsed + ` gas`,
      );
      let updateGas = new gasUsed(`Update ` + counter, result.gasUsed);
      gasArrayLightClient.push(updateGas);

      const wasmEvent = result.logs[0].events.find(e => e.type === 'wasm');
      console.info(
        'The "wasm" event emitted by the contract execution:',
        wasmEvent,
      );
    }

    // Query contract after execution ( after all updates )
    const queryResultAfterAllUpdates = await client.queryContractSmart(
      _contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterAllUpdates = byteArrayToNumber(
      queryResultAfterAllUpdates.slice(0, 8),
    );

    appendJsonFile(gasUsageFile, gasArrayLightClient);

    expect(headerSlotAfterAllUpdates).toEqual(expectedHeaderSlot);

    await stopCosmosNode('light-client');
  }, 720000);
});
