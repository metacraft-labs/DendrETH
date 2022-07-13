#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

const SCRIPT_PATH = './scripts/smartpy-cli/SmartPy.sh';
const TEST_PATH = './build/test/';
const SRC_PATH = './src';

// Change the following regex to filter the templates being tested
const FILE_FILTER = /.*[.]contract[.]ts/;

if (fs.existsSync(TEST_PATH)) {
    fs.rmSync(TEST_PATH, { recursive: true });
}
fs.mkdirSync(TEST_PATH, { recursive: true });

const files = fs.readdirSync(SRC_PATH, { encoding: 'utf-8' });

files
    .filter((f) => f.match(FILE_FILTER))
    .map(async (fileName) => {
        try {
            const dir = `${TEST_PATH}${fileName.replace('.ts', '')}`;
            fs.mkdirSync(dir, { recursive: true });
            execSync(`${SCRIPT_PATH} test ${SRC_PATH}/${fileName} ${dir}`);
        } catch (e) {
            console.error(e.output.toString() || e);
        }
    });
