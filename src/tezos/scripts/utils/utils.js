const fs = require('fs');
const path = require('path');

const getFilesInDir = function(_path) {
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

module.exports.getFilesInDir = getFilesInDir;