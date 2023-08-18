import * as winston from 'winston';

const { combine, timestamp, ms, printf } = winston.format;

export function getGenericLogger() {
  const tsFormat = printf(({ timestamp, ms, message }: any) => {
    return `[${timestamp}][${ms}] ${message}`;
  });

  const logConfiguration = {
    format: combine(
      timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
      ms(),
      tsFormat,
    ),

    transports: [new winston.transports.Console()],
    exitOnError: false,
  };

  return winston.createLogger(logConfiguration);
}
