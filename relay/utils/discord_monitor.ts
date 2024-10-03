import { ethers } from 'ethers';
import * as Discord from 'discord.js';
import contractAbi from '../../beacon-light-client/solidity/tasks/hashi_abi.json';
import { getEnvString } from '@dendreth/utils/ts-utils/common-utils';

async function getLastEventTime(
  contract: ethers.Contract,
  network: string,
): Promise<number> {
  const latestBlock = await contract.provider.getBlockNumber();
  const filter = contract.filters.HashStored();

  const events = await contract.queryFilter(
    filter,
    latestBlock - 10000,
    latestBlock,
  );
  if (events.length === 0) {
    console.log(`No events found for network '${network}'.`);
    throw new Error('No events found.');
  }

  return (await events[events.length - 1].getBlock()).timestamp * 1000;
}

// Monitor function that checks for contract updates and dispatches Discord alerts
async function checkContractUpdate(
  providerUrl: string,
  contractAddress: string,
  network: string,
  alertThresholdMinutes: number,
  discordClient: Discord.Client,
) {
  try {
    const provider = new ethers.providers.JsonRpcProvider(providerUrl);
    const contract = new ethers.Contract(
      contractAddress,
      contractAbi,
      provider,
    );

    const lastEventTime = await getLastEventTime(contract, network);
    const delayInMinutes = (Date.now() - lastEventTime) / 1000 / 60;

    if (!discordClient.isReady()) {
      console.error('Discord client is not ready.');
      return;
    }

    const channel = discordClient.channels.cache.get(
      getEnvString('CHANNEL_ID'),
    ) as Discord.TextChannel;
    if (!channel) {
      console.error('Discord channel not found.');
      return;
    }

    if (delayInMinutes >= alertThresholdMinutes) {
      const message = `<@&1285217827560620113> ⚠️ Alert: Contract on **${network}** hasn't been updated in ${delayInMinutes.toFixed(
        2,
      )} minutes.`;
      await channel.send(message);
    } else {
      console.log(
        `Contract on ${network} is up to date. Last update was before: ${delayInMinutes.toFixed(
          2,
        )} minutes`,
      );
    }
  } catch (error) {
    if (error instanceof Error) {
      console.error(
        `Error checking contract on network '${network}':`,
        error.message,
      );
    } else {
      console.error(`Unknown error occurred on network '${network}':`, error);
    }
  }
}

async function monitorContracts(networksAndAddresses: string[]) {
  const discordClient = new Discord.Client({
    intents: [
      Discord.GatewayIntentBits.Guilds,
      Discord.GatewayIntentBits.GuildMessages,
    ],
  });

  discordClient.once(Discord.Events.ClientReady, () => {
    console.log(`Logged in as ${discordClient.user?.tag}!`);
  });

  await discordClient.login(getEnvString('DISCORD_TOKEN'));

  const alertThresholdMinutes = parseInt(
    getEnvString('ALERT_THRESHOLD_MINUTES'),
    10,
  );

  const networks: string[] = [];

  for (let i = 0; i < networksAndAddresses.length; i += 2) {
    const network = networksAndAddresses[i];
    const contractAddress = networksAndAddresses[i + 1];

    try {
      const rpcUrl = getEnvString(`${network.toUpperCase()}_RPC`);

      networks.push(network);

      // Immediately check the contract update, then set interval
      checkContractUpdate(
        rpcUrl,
        contractAddress,
        network,
        alertThresholdMinutes,
        discordClient,
      );

      setInterval(
        () =>
          checkContractUpdate(
            rpcUrl,
            contractAddress,
            network,
            alertThresholdMinutes,
            discordClient,
          ),
        5 * 60 * 1000,
      );
    } catch (error) {
      if (error instanceof Error) {
        console.warn(`Skipping network '${network}' due to:`, error.message);
      } else {
        console.warn(
          `Skipping network '${network}' due to unknown error:`,
          error,
        );
      }
    }
  }

  console.log(
    `Monitoring the following networks: [ '${networks.join("', '")}' ]`,
  );
}

(async () => {
  const args = process.argv.slice(2);

  if (args.length === 0 || args.length % 2 !== 0) {
    console.error(
      'Please specify the network and contract address pairs (e.g., sepolia 0x1234 chiado 0x5678).',
    );
    process.exit(1);
  }

  await monitorContracts(args);
})();
