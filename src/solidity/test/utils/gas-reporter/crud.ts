import * as fs from "fs";
import * as path from "path";

import { Path, StringifiedContract } from "./types";

export const copyRecursiveSync = function (src: string, dest: string): Path[] {
    const paths = [];

    var exists = fs.existsSync(src);
    var stats = exists && fs.statSync(src);
    var isDirectory = exists && stats.isDirectory();
    if (isDirectory) {
        fs.mkdirSync(dest);
        fs.readdirSync(src).forEach(function (childItemName) {
            paths.push(...copyRecursiveSync(path.join(src, childItemName),
                path.join(dest, childItemName)) as never[]);
        });
    } else {
        paths.push({ src, dest } as never);
        fs.copyFileSync(src, dest);
    }

    return paths;
};

export const writeContract = (contract: StringifiedContract, dest: string): StringifiedContract => {
    fs.writeFileSync(path.join(dest), contract);
    return contract;
};