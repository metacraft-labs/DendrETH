import { ethers } from 'ethers';
import { readFileSync } from 'fs';
import { postUpdateOnChain } from './relayer-helper';

const provider = new ethers.providers.JsonRpcProvider(
  'https://opt-goerli.g.alchemy.com/v2/CQd8BYX63I75UnEaAa6-F5hlRkfheHB6',
);

const wallet = new ethers.Wallet(
  '0x7fc217a89873ddcba36225dcdaae0b631c32430bf318e665eefc7cd18dc9b19f',
  provider,
);

const light_client_abi = JSON.parse(
  readFileSync('./light_client.abi.json', 'utf-8'),
);

const lightClientContract = new ethers.Contract(
  '0x1a2FAA5f49385EebA349fd2616BAbf1Eb4367dcc',
  light_client_abi,
  wallet,
);

const proof = JSON.parse(readFileSync('proof.json', 'utf-8'));
const publicJSON = JSON.parse(readFileSync('public.json', 'utf-8'));
const proofInput = JSON.parse(readFileSync('input.json', 'utf-8'));

postUpdateOnChain(
  {
    proof: { ...proof, public: [...publicJSON] },
    proofInput: proofInput,
  } as any,
  lightClientContract,
);
