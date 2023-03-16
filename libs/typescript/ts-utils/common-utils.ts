export function byteArrayToNumber(byteArray) {
  var value = 0;
  for (var i = byteArray.length - 1; i >= 0; i--) {
    value = value * 256 + byteArray[i];
  }
  return value;
}

export function checkConfig(config: any) {
  for (const envVar of Object.keys(config)) {
    if (!config[envVar]) {
      console.warn(`$${envVar} environment variable is not set`);
      process.exit(0);
    }
  }
}
