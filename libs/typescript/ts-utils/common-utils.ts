import * as fs from 'fs';

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

// Open Json file and append data to it
export function appendJsonFile(filePath: string, data: any) {
  let fileData: any[] = [];
  try {
    fileData = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (e) {
    console.warn(`Unable to read file ${filePath}`);
  }
  fileData.push(data);
  fs.writeFileSync(filePath, JSON.stringify(fileData, null, 2));
}
