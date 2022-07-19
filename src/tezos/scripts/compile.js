#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const SCRIPT_PATH = path.join(__dirname, '..', 'scripts', 'smartpy-cli', 'SmartPy.sh');
const BUILD_PATH = path.join(__dirname, '..', 'build', 'compilation');
const SRC_PATH = path.join(__dirname, '..', 'src');

// Change the following regex to filter the templates being tested
const FILE_FILTER = /.*[.]ts/;

if (fs.existsSync(BUILD_PATH)) {
    fs.rmSync(BUILD_PATH, { recursive: true });
}
fs.mkdirSync(BUILD_PATH, { recursive: true });

function getFilesInDir(_path) {
    let files = [];
    const fileAndFolders = fs.readdirSync(_path, { encoding: 'utf-8' ,withFileTypes: true});
    for (let ff of fileAndFolders) {
        if (ff.isDirectory()) {
            files = [...files, ...getFilesInDir(path.join(_path, ff.name))]
        } else {
            files.push({
                name: ff.name,
                path: _path
            })
        }
    }
    return files;
}

const files = getFilesInDir(SRC_PATH, { encoding: 'utf-8' ,withFileTypes: true});

files
    .filter((f) => f.name.match(FILE_FILTER))
    .map(async (file) => {
        try {
            const dir = path.join(BUILD_PATH, file.name.replace('.ts', ''));
            fs.mkdirSync(dir, { recursive: true });
            console.log(`sh ${SCRIPT_PATH} compile ${file.path}/${file.name} ${dir}`)
            execSync(`sh ${SCRIPT_PATH} compile ${file.path}/${file.name} ${dir}`);
        } catch (e) {
            // console.error(e.message);
        }
    });