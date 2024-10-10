# Steps to run the relay

### 1 Check your .env you need the following set up:

#### 1.1 Must have

PROVER_SERVER_HOST<br>
PROVER_SERVER_PORT<br>
SLOTS_JUMP<br>

and files with keys at:<br>
USER_PRIVATE_KEY_FILE=/.secrets/user_private_key<br>
INFURA_API_KEY_FILE=/.secrets/infura_api_key<br>

#### 1.2 Not necessary

REDIS_HOST(if not given, will use default - localhost)<br>
REDIS_PORT(if not given, will use default - 6379)<br>

#### 1.3 Good to have

To run the relay you need `light_client.zkey` and `light_client.dat` files, if you dont have them we have provided <br>
LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION<br>
LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION

#### 1.4 Network/contract setup

you need at least one contract address<br>
example:<br>

LC_SEPOLIA=<i>contractAddress</i><br>
This is the address of the light client contract(deployed on Sepolia)<br>

SEPOLIA_HASHI=<i>contractAddress</i><br>
This is the address of the hashi contract(deployed on Sepolia)<br>

FOLLOW_NETWORK_SEPOLIA=chiado<br>
This is the network whose API, our contracts above use to update themselves<br>

BEACON_REST_API_CHIADO=<i>chiadoAPI</i><br>

### 2 in a new terminal run `yarn redis-commander`

### 3 `nix develop .#light-client`

### 4 `export LODESTAR_PRESET=gnosis`(if running on Sepolia)

### 5 `process-compose`

If you dont get updates on chain for 15+ min -> flush the redis(redis-cli FLUSHALL) and rerun the relay(or restart pollUpdateWorker task).<br>

After any changes in .env(or running direnv reload) you will need to run `nix develop .#light-client` again before starting the relay.

### Prometheus ports

pollUpdatesWorker - 2999<br>
proofGenerationWorker - 3000<br>
Sepolia - 3004<br>
Chiado - 3010<br>
Lukso - 3015
