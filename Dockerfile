git submodule update --init --recursive
yarn install --production

# Build rapidsnark
cd vendor/rapidsnark
npm install
git submodule init
git submodule update
npx task createFieldSources
npx task buildProver

# Build circuit
cd ../build-artifacts/light_client_cpp
make

cd ../../../

# Deploy the contract into Goerli
cd beacon-light-client/solidity
yarn hardhat deploy --network goerli

# Start relayer
redis-server
cd ../circom/scripts/light_client
yarn ts-node relayer.ts
cd workers
yarn ts-node get-update-workers.ts
yarn ts-node proof-generator-worker.ts
yarn ts-node publish-on-chain-worker.ts

