import * as fs from "fs";
// import * as path from "path";

import hre from "hardhat";

import { getFilesInDir } from "..";
import { RawContract, ArrayifiedContract } from "./types";
import { importHardhatConsoles } from "./log";
import { copyRecursiveSync, writeContract } from "./crud";
import { stringify } from "./format";
import { contractsOrigPath, contractsTempPath } from "./constants";

fs.rmSync(contractsTempPath, {
    recursive: true,
    force: true
});

const rawContracts: RawContract[] = getFilesInDir(contractsOrigPath);
const tempContracts: ArrayifiedContract[] = importHardhatConsoles(rawContracts);

// Attach `console.log(gasleft());` on key lines (ignore inline-assemly and for-loop bodies)
const detectKeyLines = (contracts: ArrayifiedContract[]): Set<number>[] => {
    let kl: Set<number>[] = [];
    contracts.map((contract: ArrayifiedContract) => {
        const keyLines = new Set<number>();
        // algo for finding function bodies
        for (let i = 0; i < contract.length; i++) {
            const line = contract[i];
            if (!line.trim().startsWith("function ")) continue;
            // we are inside the function declaration
            const start = line.indexOf("function ");
            for (let j = i; j < contract.length; j++) {
                if (!line.includes("{")) continue;
                if (line.includes("}")) continue; // no function body
                // we are inside the function body/scope
                keyLines.add(j + 1);
                // search for lines that end with `;` until a `{` appears
                for (let z = j + 1; z < contract.length; z++) {
                    if (line.indexOf("}") === start) break;
                    const line2 = contract[z];
                    if (line.includes("function hash_tree_root(BeaconBlockHeader memory beacon_header)")) {
                        console.log(line2);
                    }
                    let start2: undefined | number = undefined;
                    // handle loops and inline assembly (yul) - place a single console.log before and after the loop, ignore body since it will log too many times
                    if (["for (", "for(", "while (", "while(", "assembly {", "assembly{", "unchecked {", "unchecked{"].some((v) => line2.trim().startsWith(v))) {
                        // we are inside a loop 
                        keyLines.add(z - 1);
                        start2 = [line2.indexOf("for ("), line2.indexOf("for("), line2.indexOf("while ("), line2.indexOf("while("), line2.indexOf("assembly ("), line2.indexOf("assembly("), line2.indexOf("unchecked ("), line2.indexOf("unchecked(")].filter(x => x !== -1)[0];
                        for (let x = z; x < contract.length; x++) {
                            if (line2.indexOf("}") !== start2) continue;
                            // end of loop
                            keyLines.add(z + 1);
                            z = x; // continue from end of loop
                            break; // should go to `if (start2 !== undefined) continue;`
                        }
                    }
                    if (start2 !== undefined) continue;
                    if (!line2.trim().endsWith(";")) continue;
                    if (contract[z - 1].trim().startsWith("return ") || line2.trim().startsWith("return ")) continue;
                    keyLines.add(z);
                }
                break;
            }
        }
        kl.push(keyLines);
    });
    return kl;
};

const kl = detectKeyLines(tempContracts);

tempContracts.map((contract, i) => {
    const contractKeyLines = kl[i];
    for (let keyLine of contractKeyLines) {
        contract[keyLine] = contract[keyLine].concat(` //gas-reporter #${keyLine + 1}`);
    }
});

const paths = copyRecursiveSync(contractsOrigPath, contractsTempPath);
tempContracts.map(stringify)
    .map((contract, i) => writeContract(contract, paths[i].dest));

hre.run("compile",);

const gas_reporter = async (callback: () => unknown) => {
    callback();
};

export default gas_reporter;

