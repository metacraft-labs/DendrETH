import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';
import * as borsh from 'borsh';
import { getFilesInDir } from '../../../libs/typescript/ts-utils/data';
import { getPayer, getRpcUrl } from './utils';
import path from 'path';
import { formatJSONUpdate } from '../../solidity/test/utils/format';
import * as constants from '../../solidity/test/utils/constants';
import { readFileSync } from 'fs';
import { getProofInstruction } from './generate_proof_instruction';

let connection: Connection;

let payer: Keypair;

class LightClientAccount {
  public_finalized_header_root = new Uint8Array(32);
  prev_block_header_root = new Uint8Array(32);
  constructor(
    fields:
      | {
          public_finalized_header_root: Uint8Array;
          prev_block_header_root: Uint8Array;
        }
      | undefined = undefined,
  ) {
    if (fields) {
      this.public_finalized_header_root = fields.public_finalized_header_root;
      this.prev_block_header_root = fields.prev_block_header_root;
    }
  }
}

/**
 * Borsh schema definition for greeting accounts
 */
const LightClientAccountgSchema = new Map([
  [
    LightClientAccount,
    {
      kind: 'struct',
      fields: [
        ['public_finalized_header_root', [32]],
        ['prev_block_header_root', [32]],
      ],
    },
  ],
]);

/**
 * Light client program Id
 */
let programId: PublicKey = new PublicKey(
  'BFMLN7HAJiMjZcnADKFgcRGqfPTGuR7ETcNteydamJs9',
);

let light_client_PubKey: PublicKey;

/**
 * The expected size of each greeting account.
 */
const GREETING_SIZE = borsh.serialize(
  LightClientAccountgSchema,
  new LightClientAccount(),
).length;

async function main() {
  console.log("Let's test it...");

  await establishConnection();

  await establishPayer();

  await verifyProof();

  console.log('Success');
}

async function verifyProof(): Promise<void> {
  for (let i = 291; i < 292; i++) {
    const proof = JSON.parse(
      readFileSync(
        path.join(
          __dirname,
          `../../../vendor/eth2-light-client-updates/mainnet/proofs/proof${i}.json`,
        ),
      ).toString(),
    );
    const public_data = JSON.parse(
      readFileSync(
        path.join(
          __dirname,
          `../../../vendor/eth2-light-client-updates/mainnet/proofs/public${i}.json`,
        ),
      ).toString(),
    );

    const byte_array = getProofInstruction(proof, public_data);

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: light_client_PubKey, isSigner: false, isWritable: true },
      ],
      programId,
      data: Buffer.from(byte_array),
    });

    await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [payer],
    );
  }
}

async function establishConnection(): Promise<void> {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
}

async function establishPayer(): Promise<void> {
  let fees = 0;
  if (!payer) {
    const { feeCalculator } = await connection.getRecentBlockhash();

    // Calculate the cost to fund the greeter account
    fees += await connection.getMinimumBalanceForRentExemption(GREETING_SIZE);

    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    payer = await getPayer();
  }

  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop

    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    'Using account',
    payer.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );

  const SEED = 'hello';
  light_client_PubKey = await PublicKey.createWithSeed(
    payer.publicKey,
    SEED,
    programId,
  );

  // Check if the greeting account has already been created
  const lightClientAccount = await connection.getAccountInfo(
    light_client_PubKey,
  );
  if (lightClientAccount === null) {
    console.log(
      'Creating account',
      light_client_PubKey.toBase58(),
      'for light client state',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      GREETING_SIZE,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: SEED,
        newAccountPubkey: light_client_PubKey,
        lamports,
        space: GREETING_SIZE,
        programId,
      }),
    );
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }
}

main();
