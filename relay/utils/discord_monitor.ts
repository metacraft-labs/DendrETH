import { SolidityContract } from '../implementations/solidity-contract';
import { getBeaconApi, BeaconApi } from '../implementations/beacon-api';
import { ethers } from 'ethers';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import { GatewayIntentBits, Events, Partials } from 'discord.js';
import * as Discord from 'discord.js';
import lc_abi_json from '../../beacon-light-client/solidity/artifacts/contracts/bridge/src/truth/eth/BeaconLightClient.sol/BeaconLightClient.json';

const env = process.env;

interface ContractData {
  RPC: string;
  Address: string;
  SolidityContract?: SolidityContract;
}

type SolidityDictionary = {
  [name: string]: ContractData;
};

class DiscordMonitor {
  private readonly contracts: SolidityDictionary = {};

  constructor(
    private readonly client: Discord.Client,
    private readonly beaconApi: BeaconApi,
    private readonly alert_threshold: number,
  ) {
    if (env.GOERLI_RPC && env.LC_GOERLI) {
      this.contracts['Goerli'] = {
        RPC: env.GOERLI_RPC,
        Address: env.LC_GOERLI,
      };
    }
    if (env.OPTIMISTIC_GOERLI_RPC && env.LC_OPTIMISTIC_GOERLI) {
      this.contracts['OptimisticGoerli'] = {
        RPC: env.OPTIMISTIC_GOERLI_RPC,
        Address: env.LC_OPTIMISTIC_GOERLI,
      };
    }
    if (env.BASE_GOERLI_RPC && env.LC_BASE_GOERLI) {
      this.contracts['BaseGoerli'] = {
        RPC: env.BASE_GOERLI_RPC,
        Address: env.LC_BASE_GOERLI,
      };
    }

    if (env.ARBITRUM_GOERLI_RPC && env.LC_ARBITRUM_GOERLI) {
      this.contracts['ArbitrumGoerli'] = {
        RPC: env.ARBITRUM_GOERLI_RPC,
        Address: env.LC_ARBITRUM_GOERLI,
      };
    }
    if (env.SEPOLIA_RPC && env.LC_SEPOLIA) {
      this.contracts['Sepolia'] = {
        RPC: env.SEPOLIA_RPC,
        Address: env.LC_SEPOLIA,
      };
    }
    if (env.MUMBAI_RPC && env.LC_MUMBAI) {
      this.contracts['Mumbai'] = {
        RPC: env.MUMBAI_RPC,
        Address: env.LC_MUMBAI,
      };
    }
    if (env.FANTOM_RPC && env.LC_FANTOM) {
      this.contracts['Fantom'] = {
        RPC: env.FANTOM_RPC,
        Address: env.LC_FANTOM,
      };
    }
    if (env.CHIADO_RPC && env.LC_CHIADO) {
      this.contracts['Chiado'] = {
        RPC: env.CHIADO_RPC,
        Address: env.LC_CHIADO,
      };
    }
    if (env.GNOSIS_RPC && env.LC_GNOSIS) {
      this.contracts['Gnosis'] = {
        RPC: env.GNOSIS_RPC,
        Address: env.LC_GNOSIS,
      };
    }
    if (env.BSC_RPC && env.LC_BSC) {
      this.contracts['BSC'] = { RPC: env.BSC_RPC, Address: env.LC_BSC };
    }
    if (env.AURORA_RPC && env.LC_AURORA) {
      this.contracts['Aurora'] = {
        RPC: env.AURORA_RPC,
        Address: env.LC_AURORA,
      };
    }

    // Instantiate SolidityContracts from .env
    for (let endpoint in this.contracts) {
      let curLightClient = new ethers.Contract(
        this.contracts[endpoint].Address,
        lc_abi_json.abi,
        new ethers.providers.JsonRpcProvider(this.contracts[endpoint].RPC), // Provider
      );

      let curSolidityContract = new SolidityContract(
        curLightClient,
        this.contracts[endpoint].RPC,
      );
      this.contracts[endpoint].SolidityContract = curSolidityContract;
    }
  }

  public static async initializeDiscordMonitor(
    alert_threshold: number,
  ): Promise<DiscordMonitor> {
    const beaconApi = await getBeaconApi([
      'http://unstable.prater.beacon-api.nimbus.team/',
    ]);

    const client = new Discord.Client({
      intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.GuildMembers,
        GatewayIntentBits.DirectMessages,
      ],
      partials: [Partials.Channel, Partials.Message, Partials.Reaction],
    });

    const result = await client.login(env.token);

    await client.on(Events.ClientReady, async interaction => {
      console.log('Client Ready!');
      console.log(`Logged in as ${client.user?.tag}!`);
    });

    return new DiscordMonitor(client, beaconApi, alert_threshold);
  }

  private async getSlotDelay(contract: SolidityContract) {
    return (
      (await this.beaconApi.getCurrentHeadSlot()) -
      (await contract.optimisticHeaderSlot())
    );
  }

  private async respondToMessage() {
    //TODO: Implement responsive commands
    this.client.on(Events.MessageCreate, message => {
      if (message.author.bot) return;

      // Nice to have, responsive bot
      console.log(
        `Message from ${message.author.username}: ${message.content}`,
      );
      if (message.content === '') console.log('Empty message'); //TODO: Bot can't read user messages
    });
  }

  public async dispatchMessage(messageToSend) {
    let channel = this.client.channels.cache.get(
      env.channel_id!,
    ) as Discord.TextChannel;
    if (!channel) {
      channel = (await this.client.channels.fetch(
        env.channel_id!,
      )) as Discord.TextChannel;
    }

    await channel.send(messageToSend);
  }

  public async monitor_delay() {
    for (let contract of Object.keys(this.contracts)) {
      let name = contract;
      let delay = await this.getSlotDelay(
        this.contracts[contract].SolidityContract!,
      );

      // Dispatch
      const minutes_delay = (delay * 1) / 5;
      if (minutes_delay >= this.alert_threshold || delay < 0) {
        let message = `Contract: ${name} is behind Beacon Head with ${minutes_delay} minutes`;
        this.dispatchMessage(message);
      }
    }
  }
}

(async () => {
  let monitor = await DiscordMonitor.initializeDiscordMonitor(
    Number(env.ping_threshold),
  );

  monitor.dispatchMessage('Relayer bot starting!');

  let retry_counter = 0;
  while (true) {
    if (retry_counter >= 10) {
      throw new Error(
        `Failed connection to Discord after ${retry_counter} retries`,
      );
    }

    const msTimeout = 10_000;
    let waitPromise = new Promise<'timeout'>(resolve =>
      setTimeout(() => resolve('timeout'), msTimeout),
    );
    let response = await Promise.race([monitor.monitor_delay(), waitPromise]);

    retry_counter += response == 'timeout' ? 1 : 0;

    await sleep(env.ping_timeout);
  }
})();
