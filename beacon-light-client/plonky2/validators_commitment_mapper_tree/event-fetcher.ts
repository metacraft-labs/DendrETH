import { ethers, Contract } from 'ethers';

// get events from a contract in a given block range
export const getEvents = async (
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
