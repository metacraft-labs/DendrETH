// import { Queue } from "bullmq";
// import { GetUpdate } from "../types/types";
// import { UPDATE_POLING_QUEUE } from "../constants/constants";
// import { addUpdate } from "../utils/orchestrator";
// import { getNetworkConfig } from "../utils/get_current_network_config";

// (async () => {
//   const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
//     connection: {
//       host: process.env.REDIS_HOST!,
//       port: Number(process.env.REDIS_PORT),
//     },
//   });

//   const networkConfig = getNetworkConfig('pratter');

//   while (await addUpdate(
//     6020610,
//     32,
//     6021156,
//     updateQueue,
//     networkConfig
//   )) { }
// })();
