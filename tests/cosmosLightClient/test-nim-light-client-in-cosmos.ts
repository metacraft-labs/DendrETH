import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readFile } from 'fs/promises';
import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';
import { SSZSpecTypes } from '../../libs/typescript/ts-utils/sszSpecTypes.js';
import { jsonToSerializedBase64 } from '../../libs/typescript/ts-utils/ssz-utils';
import { compileNimFileToWasm } from '../../libs/typescript/ts-utils/compile-nim-to-wasm.js';

const exec = promisify(exec_);

describe('Light Client In Cosmos', () => {
  const controller = new AbortController();
  const { signal } = controller;

  const rootDir = dirname(fileURLToPath(import.meta.url));

  const rpcEndpoint = 'http://localhost:26657';
  const gasPrice = GasPrice.fromString('0.0000025ustake');

  let DendrETHWalletInfo = {
    mnemonic:
      'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone',
    address: '',
  };

  let wallet: OfflineSigner, client: SigningCosmWasmClient;
  let _contractAddress;
  beforeAll(async () => {
    let contractDir = rootDir + `/../../contracts/cosmos/light-client`;
    let nimFilePath = contractDir + `/lib/nim/light_client_cosmos_wrapper.nim`;
    await compileNimFileToWasm(
      nimFilePath,
      `--nimcache:${contractDir}/nimbuild --d:lightClientCosmos -o:${contractDir}/nimbuild/light_client.wasm`,
    );

    let compileContractCommand = `docker run -t --rm -v "${contractDir}":/code --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/rust-optimizer:0.12.8 .`;
    console.info(`➤ ${compileContractCommand}`);
    await exec(compileContractCommand);

    const setupWasmdCommand = `bash ${rootDir}/../../contracts/cosmos/light-client/scripts/setup_wasmd.sh`;
    console.info(`➤ ${setupWasmdCommand}`);
    execSync(setupWasmdCommand);

    const startNodeCommand = `bash ${rootDir}/../../contracts/cosmos/light-client/scripts/start_node.sh`;
    console.info(`➤ ${startNodeCommand}`);
    exec_(startNodeCommand, { signal });

    execSync('sleep 10');

    const addKeyCommand = `bash ${rootDir}/../../contracts/cosmos/light-client/scripts/add_account.sh`;
    console.info(`➤ ${addKeyCommand}`);
    execSync(addKeyCommand);

    const getFredAddressCommand = `wasmd keys show fred -a`;
    console.info(`➤ ${getFredAddressCommand}`);
    const getAddress = exec(getFredAddressCommand);
    const fredDrres = (await getAddress).stdout;

    DendrETHWalletInfo.address = fredDrres.trimEnd();
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      DendrETHWalletInfo.mnemonic,
      {
        prefix: 'wasm',
      },
    );
    client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet);
  }, 360000 /* timeout in milliseconds */);

  test('Check "LightClientStore" after initialization', async () => {
    // The contract
    const wasm = fs.readFileSync(
      rootDir +
        `/../../contracts/cosmos/light-client/artifacts/light_client.wasm`,
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

    // Instantiate the contract
    const instantiateFee = calculateFee(12_500_000, gasPrice);
    const bootstrapData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientBootstrap,
      rootDir +
        `/../../vendor/eth2-light-client-updates/mainnet/bootstrap.json`,
    );

    // This contract specific message is passed to the contract
    const msg = {
      bootstrap_data: bootstrapData,
    };
    const { contractAddress } = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      msg,
      'My instance',
      instantiateFee,
      { memo: 'Create a Cosmos Light Clinet instance.' },
    );
    console.info('Contract instantiated at: ', contractAddress);
    _contractAddress = contractAddress;

    // Query contract after initialization
    const queryResultAfterInitialization = await client.queryContractSmart(
      contractAddress,
      {
        store: {},
      },
    );

    const expectedStoreDataAfterInitialization = await readFile(
      rootDir + `/data/store_data_on_initialization.txt`,
      'utf-8',
    );

    expect(String(queryResultAfterInitialization)).toEqual(
      expectedStoreDataAfterInitialization.trimEnd(),
    );
  }, 300000);

  test('Check "LightClientStore" after one update', async () => {
    const updateData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientUpdate,
      rootDir +
        `/../../vendor/eth2-light-client-updates/mainnet/updates/00290.json`,
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

    const expectedStoreDataAfterOneUpdate = await readFile(
      rootDir + `/data/store_data_after_first_update.txt`,
      'utf-8',
    );

    expect(String(queryResultAfterOneUpdate)).toEqual(
      expectedStoreDataAfterOneUpdate.trimEnd(),
    );
  }, 300000);

  test('Check "LightClientStore" after all updates', async () => {
    const updateFiles = glob(
      rootDir +
        `/../../vendor/eth2-light-client-updates/mainnet/updates/*.json`,
    );
    for (var updateFile of updateFiles) {
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

    const expectedStoreDataAfterAllUpdates = await readFile(
      rootDir + `/data/store_data_after_all_updates.txt`,
      'utf-8',
    );

    expect(String(queryResultAfterAllUpdates)).toEqual(
      expectedStoreDataAfterAllUpdates.trimEnd(),
    );
    controller.abort();
  }, 1500000);
});
