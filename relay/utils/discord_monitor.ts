import { SolidityContract } from '../implementations/solidity-contract';
import { BeaconApi } from '../implementations/beacon-api';
import { ethers } from "ethers";
import {sleep} from '../../libs/typescript/ts-utils/common-utils';
import lc_abi_json from '../../beacon-light-client/solidity/artifacts/contracts/bridge/src/truth/eth/BeaconLightClient.sol/BeaconLightClient.json';

const env = require('dotenv').config({path: '../../.env'}).parsed;

const { GatewayIntentBits, SlashCommandBuilder, Events,Partials } = require('discord.js');
const Discord = require('discord.js');

interface ContractData {
    RPC: string;
    Address: string;
    SolidityContract?: SolidityContract;
}

type SolidityDictionary = {
    [name: string]: ContractData;
};

class DiscordMonitor {
    client: any;
    beaconApi: BeaconApi;
    contracts: SolidityDictionary = {};

    alert_threshold: number;
     
    constructor(alert_threshold: number) {

        this.alert_threshold = alert_threshold;

        this.beaconApi = new BeaconApi([
            'http://unstable.prater.beacon-api.nimbus.team/',
          ]);
    
        this.client = new Discord.Client({
        intents: [
            GatewayIntentBits.Guilds,
            GatewayIntentBits.GuildMessages,
            GatewayIntentBits.GuildMembers,
            GatewayIntentBits.DirectMessages
            ],
            partials: [
            Partials.Channel,
            Partials.Message,
            Partials.DirectMessages,
            Partials.Reaction
            ]
        });

        this.client.login(env.token);
        this.client.on(Events.ClientReady, async interaction => {
            console.log('Client Ready!')
            console.log(`Logged in as ${this.client.user.tag}!`) 
        });

        // Track contracts if initialized in .env
        if (env.GOERLI_RPC && env.LC_GOERLI){
            this.contracts['Goerli'] =  {RPC: env.GOERLI_RPC, Address: env.LC_GOERLI};
        }
        if (env.OPTIMISTIC_GOERLI_RPC && env.LC_OPTIMISTIC_GOERLI) {
            this.contracts['OptimisticGoerli'] = {RPC: env.OPTIMISTIC_GOERLI_RPC, Address: env.LC_OPTIMISTIC_GOERLI};
        }
        if (env.BASE_GOERLI_RPC && env.LC_BASE_GOERLI) {
            this.contracts['BaseGoerli'] = {RPC: env.BASE_GOERLI_RPC, Address: env.LC_BASE_GOERLI};
        }
        if (env.ARBITRUM_GOERLI_RPC && env.LC_ARBITRUM_GOERLI) {
            this.contracts['ArbitrumGoerli'] = {RPC: env.ARBITRUM_GOERLI_RPC, Address: env.LC_ARBITRUM_GOERLI};
        }
        if (env.SEPOLIA_RPC && env.LC_SEPOLIA) {
            this.contracts['Sepolia'] = {RPC: env.SEPOLIA_RPC, Address: env.LC_SEPOLIA};
        }
        if (env.MUMBAI_RPC && env.LC_MUMBAI) {
            this.contracts['Mumbai'] = {RPC: env.MUMBAI_RPC, Address: env.LC_MUMBAI};
        }
        if (env.FANTOM_RPC && env.LC_FANTOM) {
            this.contracts['Fantom'] = {RPC: env.FANTOM_RPC, Address: env.LC_FANTOM};
        }
        if (env.CHIADO_RPC && env.LC_CHIADO) {
            this.contracts['Chiado'] = {RPC: env.CHIADO_RPC, Address: env.LC_CHIADO};
        }
        if (env.GNOSIS_RPC && env.LC_GNOSIS) {
            this.contracts['Gnosis'] = {RPC: env.GNOSIS_RPC, Address: env.LC_GNOSIS};
        }
        if (env.BSC_RPC && env.LC_BSC) {
            this.contracts['BSC'] = {RPC: env.BSC_RPC, Address: env.LC_BSC};
        }
        if (env.AURORA_RPC && env.LC_AURORA) {
            this.contracts['Aurora'] = {RPC: env.AURORA_RPC, Address: env.LC_AURORA};
        }
    
        // Instantiate SolidityContracts from .env 
        for (var endpoint in this.contracts) {

            var curLightClient = new ethers.Contract(
                this.contracts[endpoint].Address,
                lc_abi_json.abi,
                new ethers.providers.JsonRpcProvider(this.contracts[endpoint].RPC) // Provider
            );

            var curSolidityContract = new SolidityContract(
                curLightClient,
                this.contracts[endpoint].RPC,
            );
            this.contracts[endpoint].SolidityContract= curSolidityContract;
        }
    }

    private async getSlotDelay(contract: SolidityContract) {
        return await this.beaconApi.getCurrentHeadSlot() - await contract.optimisticHeaderSlot();
    }

    private async respondToMessage() { //TODO: Implement responsive commands
        this.client.on(Events.MessageCreate, (message) => {
            if (message.author.bot) return false; 

                // Nice to have, responsive bot
                console.log(`Message from ${message.author.username}: ${message.content}`);
                if (message.content === '') console.log('Empty message') //TODO: Bot can't read user messages
        });
    }

    private async dispatchMessage(messageToSend) {
        this.client.channels.cache.get(env.channel_id).send(messageToSend)
    }

    public async monitor_delay() {

        for (var contract of Object.keys(this.contracts)) {
            var name = contract;
            var delay = await this.getSlotDelay(this.contracts[contract].SolidityContract!)

            // Dispatch
            var minutes_delay = delay * 1 / 5;

            if (minutes_delay >= this.alert_threshold || delay < 0) {
                var message = `Contract: ${ name } is behind Beacon Head with ${ minutes_delay } minutes`
                this.dispatchMessage(message);
            }
        }
    }
}

(async () => {
    var monitor = new DiscordMonitor(env.ping_threshold);
    while(true) {
        await monitor.monitor_delay();
        await sleep(env.ping_timeout);
    }
})();

