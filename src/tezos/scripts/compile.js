#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const SCRIPT_PATH = path.join(__dirname, '..', 'scripts', 'smartpy-cli', 'SmartPy.sh');
const BUILD_PATH = path.join(__dirname, '..', 'build', 'compilation');
const SRC_PATH = path.join(__dirname, '..', 'src');

// Change the following regex to filter the templates being tested
const FILE_FILTER = /.*[.]contract[.]ts/;

if (fs.existsSync(BUILD_PATH)) {
    fs.rmSync(BUILD_PATH, { recursive: true });
}
fs.mkdirSync(BUILD_PATH, { recursive: true });

const files = fs.readdirSync(SRC_PATH, { encoding: 'utf-8' });

files
    .filter((f) => f.match(FILE_FILTER))
    .map(async (fileName) => {
        try {
            const dir = path.join(BUILD_PATH, fileName.replace('.ts', ''));
            fs.mkdirSync(dir, { recursive: true });
            execSync(`sh ${SCRIPT_PATH} compile ${SRC_PATH}/${fileName} ${dir}`);
        } catch (e) {
            console.error(e.output?.toString() || e);
        }
    });
