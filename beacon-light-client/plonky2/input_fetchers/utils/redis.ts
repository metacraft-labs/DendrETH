import Redis from 'ioredis';

export default function makeRedis(options: any): Redis {
  const auth: string = '' + options['redis-auth'];
  const at: string = auth && auth.length > 0 ? '@' : '';
  const host: string = '' + options['redis-host'];
  const port: string = '' + options['redis-port'];
  const url: string = `redis://${auth}${at}${host}:${port}`;
  return new Redis(url);
}
