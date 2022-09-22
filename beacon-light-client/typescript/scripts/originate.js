#!/usr/bin/env node

const { execSync } = require('child_process');
const path = require('path');

const SCRIPT_PATH = path.join(__dirname, '..', 'scripts', 'smartpy-cli', 'SmartPy.sh');

const args = process.argv.join(' ');

var CMD = `${SCRIPT_PATH} originate-contract`;

try {
    console.log();
    console.log(execSync(`${CMD} ${args}`).toString());
    console.log();
} catch (e) {
    console.error(e);
}
