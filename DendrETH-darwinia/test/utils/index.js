const fs = require("fs");
const path = require("path");

function getFilesInDir(_path) {
    let files = [];
    const content = fs.readdirSync(_path, { encoding: 'utf-8', withFileTypes: true });
    for (let f of content) {
        if (f.isDirectory()) {
            files = [...files, ...getFilesInDir(path.join(_path, f.name))];
        } else {
            files.push(fs.readFileSync(path.join(_path, f.name)));
        }
    }
    return files;
}

module.exports = { getFilesInDir };