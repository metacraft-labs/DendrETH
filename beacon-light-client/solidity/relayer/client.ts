import { createClient } from 'redis';

const redisClient = createClient();

redisClient.connect();

export default redisClient;
