import { ethers, Contract } from 'ethers';
import ValidatorsAccumulator from '../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';

// fetch events in a given block range for a given ValidatorsAccumulator contract
const fetchEvents = async (
  provider: ethers.providers.JsonRpcProvider,
  validatorsAccumulatorAddress: string,
  fromBlock: number,
  toBlock: number,
): Promise<any> => {
  const validatorsAccumulator = await getValidatorsAccumulatorContract(
    provider,
    validatorsAccumulatorAddress,
  );

  const logs = await getEvents(
    provider,
    validatorsAccumulator,
    {
      Deposited: [
        'pubkey',
        'withdrawalCredentials',
        'signature',
        'depositMessageRoot',
        'depositDataRoot',
      ],
    },
    fromBlock,
    toBlock,
  );

  return {
    accumulator: await validatorsAccumulator.get_validators_accumulator(),
    logs,
  };
};

// get the ValidatorsAccumulator contract object
const getValidatorsAccumulatorContract = async (
  provider: ethers.providers.JsonRpcProvider,
  validatorsAccumulatorAddress: string,
): Promise<Contract> => {
  return new Contract(
    validatorsAccumulatorAddress,
    ValidatorsAccumulator.abi,
    provider,
  );
};

// get events from a contract in a given block range
const getEvents = async (
  provider: ethers.providers.JsonRpcProvider,
  contract: Contract,
  topicData: any,
  fromBlock: number,
  toBlock: number,
) => {
  const topics = Object.keys(topicData).map(
    eventName => contract.filters[eventName]().topics![0],
  );

  const filter: ethers.providers.Filter = {
    topics,
    address: contract.address,
    fromBlock,
    toBlock,
  };

  const eventLogs = await provider.getLogs(filter);

  const events = eventLogs.map((log: ethers.providers.Log) => {
    try {
      const parsedLog = contract.interface.parseLog(log)!;
      if (topics.includes(parsedLog.topic)) {
        const parsedLogData: any = {
          blockNumber: log.blockNumber,
        };
        topicData[parsedLog.name].forEach((field: string) => {
          parsedLogData[field] = parsedLog.args[field].toString();
        });
        return {
          [parsedLog.name]: parsedLogData,
        };
      }
    } catch (error) {
      console.log('error');
      if (log?.topics[0] === topics[0]) {
        console.log('log', log);
      }
    }
    return '';
  });

  return events;
};

// example usage
(async () => {
  try {
    const provider = new ethers.providers.JsonRpcProvider(
      // process.env.ETHEREUM_MAINNET_RPC,
      'http://127.0.0.1:8545/',
    );

    const validatorsAccumulatorAddress =
      '0xE2b5bDE7e80f89975f7229d78aD9259b2723d11F';
    const fromBlock = 17578100;
    const toBlock = 17578110;

    const data = await fetchEvents(
      provider,
      validatorsAccumulatorAddress,
      fromBlock,
      toBlock,
    );
    data.logs.map((log: any) => {
      console.log(log);
    });

    process.exit(0);
  } catch (e: any) {
    console.log(e.message);
    process.exit(1);
  }
})();
