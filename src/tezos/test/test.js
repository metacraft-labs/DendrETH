const { digest } = require("@chainsafe/as-sha256");
const { expect } = require('chai');
const axios = require("axios");
const {
    Sp,
    Constants: ConstantsContract,
    Helpers: HelpersContract,
    Utils: UtilsContract,
    BeaconLightClient: BeaconLightClientContract,
} = require('./implementation');


// helper for digest and sha256
function Uint8ArrayToHexString(array) {
    let hex = "";
    for (let n of array) {
        hex = hex.concat(n.toString(16).padStart(2, "0"));
    }
    return "0x".concat(hex);
}

function hexStringToUint8Array(_hex) {
    _hex = _hex.replace("0x", "");
    let array = [];
    for (let i = 0; i < _hex.length; i += 2)
        array.push(parseInt(_hex.slice(i, i + 2), 16));
    return Buffer.from(array);
}

function formatUpdate(update) {
    update.attested_header.slot = Number(update.attested_header.slot);
    update.attested_header.proposer_index = Number(update.attested_header.proposer_index);
    update.attested_header.parent_root = hexStringToUint8Array(update.attested_header.parent_root);
    update.attested_header.state_root = hexStringToUint8Array(update.attested_header.state_root);
    update.attested_header.body_root = hexStringToUint8Array(update.attested_header.body_root);

    update.next_sync_committee.pubkeys = update.next_sync_committee.pubkeys.map(hexStringToUint8Array);
    update.next_sync_committee.aggregate_pubkey = hexStringToUint8Array(update.next_sync_committee.aggregate_pubkey);

    update.next_sync_committee_branch = update.next_sync_committee_branch.map(hexStringToUint8Array);

    update.finalized_header.slot = Number(update.finalized_header.slot);
    update.finalized_header.proposer_index = Number(update.finalized_header.proposer_index);
    update.finalized_header.parent_root = hexStringToUint8Array(update.finalized_header.parent_root);
    update.finalized_header.state_root = hexStringToUint8Array(update.finalized_header.state_root);
    update.finalized_header.body_root = hexStringToUint8Array(update.finalized_header.body_root);

    update.finality_branch = update.finality_branch.map(hexStringToUint8Array);

    update.sync_aggregate.sync_committee_bits = [...hexStringToUint8Array(update.sync_aggregate.sync_committee_bits)].map(x => x.toString(2).padStart(8, "0")).join("").split("").map(Number);
    update.sync_aggregate.sync_committee_signature = hexStringToUint8Array(update.sync_aggregate.sync_committee_signature);

    update.signature_slot = Number(update.signature_slot);

    return update;
}

function formatSnapshot(snapshot) {
    snapshot.header.slot = Number(snapshot.header.slot);
    snapshot.header.proposer_index = Number(snapshot.header.proposer_index);
    snapshot.header.parent_root = hexStringToUint8Array(snapshot.header.parent_root);
    snapshot.header.state_root = hexStringToUint8Array(snapshot.header.state_root);
    snapshot.header.body_root = hexStringToUint8Array(snapshot.header.body_root);

    snapshot.current_sync_committee.pubkeys = snapshot.current_sync_committee.pubkeys.map(hexStringToUint8Array);
    snapshot.current_sync_committee.aggregate_pubkey = hexStringToUint8Array(snapshot.current_sync_committee.aggregate_pubkey);

    snapshot.current_sync_committee_branch = snapshot.current_sync_committee_branch.map(hexStringToUint8Array);

    return snapshot;
}

describe("Tests", function () {
    const contracts = {};

    const RANDOM_BYTES = [
        [0xb1, 0x0e, 0x5f, 0x99, 0xe8, 0xf9, 0xa0, 0xff, 0x42, 0x63, 0x06, 0x54, 0x1b, 0x60, 0x6e, 0x55, 0xc5, 0xcb, 0x55, 0x51, 0xe6, 0xe5, 0x55, 0x28, 0x00, 0x60, 0xad, 0x1a, 0x36, 0x61, 0x96, 0x74],
        [0x41, 0xea, 0xb3, 0xe7, 0xa7, 0x06, 0x98, 0x9d, 0x60, 0xbd, 0x4a, 0x07, 0x9f, 0x42, 0xc6, 0x62, 0xb2, 0xee, 0xae, 0x2e, 0xc4, 0xdd, 0x11, 0xeb, 0x69, 0x13, 0x9a, 0x90, 0x7f, 0x7f, 0xbb, 0x53],
        [0x94, 0x25, 0xfe, 0x42, 0xbf, 0xc1, 0xf4, 0x18, 0xce, 0xff, 0x56, 0x07, 0x8a, 0x25, 0x84, 0x90, 0x43, 0x68, 0xb5, 0x49, 0x25, 0xf4, 0xbf, 0x00, 0x8a, 0x3d, 0x76, 0x9e, 0x5b, 0xb9, 0xc5, 0x91],
        [0x3e, 0xcb, 0xf8, 0x36, 0xb6, 0xf2, 0xdc, 0xa2, 0x49, 0xf9, 0x60, 0x41, 0xaa, 0x1d, 0x6d, 0xc3, 0xd2, 0x6b, 0x34, 0x9c, 0x35, 0x41, 0xff, 0x85, 0x85, 0x02, 0x5b, 0x3e, 0x5f, 0x81, 0x97, 0x30]
    ];
    const RANDOM_BYTES_ROOT = [52, 102, 42, 239, 137, 145, 193, 146, 121, 211, 207, 70, 228, 246, 164, 172, 174, 157, 201, 52, 174, 119, 148, 78, 75, 151, 31, 52, 86, 227, 4, 92];

    const EMPTY_BYTES4 = Array(4).fill(0);
    const EMPTY_BYTES32 = Array(32).fill(0);
    const EMPTY_BYTES46 = Array(46).fill(0);

    const EMPTY_BEACON_HEADER = {
        slot: 0,
        proposer_index: 0,
        parent_root: EMPTY_BYTES32,
        state_root: EMPTY_BYTES32,
        body_root: EMPTY_BYTES32,
    };

    const EMPTY_SYNC_COMMITTEE = {
        pubkeys: [],
        aggregate_pubkey: EMPTY_BYTES46,
    };

    const EMPTY_LIGHT_CLIENT_UPDATE = {
        header: EMPTY_BEACON_HEADER,
        next_sync_committee: EMPTY_SYNC_COMMITTEE,
        next_sync_committee_branch: [],
        finality_header: EMPTY_BEACON_HEADER,
        finality_branch: [],
        sync_committee_bits: [],
        sync_committee_signature: EMPTY_BYTES46,
        fork_version: EMPTY_BYTES4,
    };

    before(async function () {
        console.log(" >>> Fetching BLC data...");
        BLC_SNAPSHOT = formatSnapshot((await axios.get("https://raw.githubusercontent.com/metacraft-labs/eth2-light-client-updates/main/mainnet/snapshot.json")).data.data.v);
        const promises = [];
        for (let i = 291; i <= 533; i++)
            promises.push(axios.get(`https://raw.githubusercontent.com/metacraft-labs/eth2-light-client-updates/main/mainnet/updates/${i.toString().padStart(5, "0")}.json`));
        BLC_UPDATES = (await Promise.all(promises)).map(x => x.data).map(formatUpdate);
    });


    describe("SP", function () {
        describe("Sp.failWith", function () {
            it("should throw an error when invoked", function () {
                expect(() => Sp.failWith()).to.throw();
            });

            it("should throw an error with the given message", function () {
                expect(() => Sp.failWith("error_message")).to.throw("error_message");
            });
        });

        describe("Sp.ediv", function () {
            it("should provide whole integer devision as first option in result", function () {
                expect(Sp.ediv(5, 2).openSome().fst()).to.eq(2);
            });

            it("should provide remainder devision as second option in result", function () {
                expect(Sp.ediv(5, 2).openSome().snd()).to.eq(1);
            });
        });

        describe("Sp.pack", function () {
            it("should parse number into uint8 array buffer", function () {
                expect(Uint8ArrayToHexString(Sp.pack(0)))
                    .to.eq(Uint8ArrayToHexString([0]));

                expect(Uint8ArrayToHexString(Sp.pack(parseInt(72057594037927936n))))
                    .to.eq("0x0100000000000000");
            });
        });

        describe("Sp.sha256", function () {
            it("should hash the passed value using sha256 algorythm", function () {
                expect(Uint8ArrayToHexString(Sp.sha256(EMPTY_BYTES32)))
                    .to.equal("0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925");
            });
        });
    });

    describe("CONSTANTS", function () {
        before(function () {
            contracts.Constants = new ConstantsContract();
        });

        describe("PHASE 0 constants", function () {
            it("DOMAIN_SYNC_COMMITTEE should be 0x07000000", function () {
                expect(contracts.Constants.DOMAIN_SYNC_COMMITTEE.length).to.eq(4);
                expect(Uint8ArrayToHexString(contracts.Constants.DOMAIN_SYNC_COMMITTEE)).to.equal("0x07000000");
            });

            it("GENESIS_FORK_VERSION should be 4 empty bytes long", function () {
                expect(contracts.Constants.GENESIS_FORK_VERSION.length).to.eq(4);
                expect(Uint8ArrayToHexString(contracts.Constants.GENESIS_FORK_VERSION)).to.equal(Uint8ArrayToHexString(EMPTY_BYTES4));
            });

            it("EPOCHS_PER_SYNC_COMMITTEE_PERIOD should be 2**8", function () {
                expect(contracts.Constants.EPOCHS_PER_SYNC_COMMITTEE_PERIOD).to.equal(2 ** 8);
            });

            it("SLOTS_PER_EPOCH should be 2**5", function () {
                expect(contracts.Constants.SLOTS_PER_EPOCH).to.equal(2 ** 5);
            });

            it("BLSPUBLICKEY_LENGTH should be 48", function () {
                expect(contracts.Constants.BLSPUBLICKEY_LENGTH).to.equal(48);
            });

            it("SYNC_COMMITTEE_SIZE should be 2**9", function () {
                expect(contracts.Constants.SYNC_COMMITTEE_SIZE).to.equal(2 ** 9);
            });
        });

        describe("ALTAIR upgrade constants", function () {
            it("FINALIZED_ROOT_INDEX should be 105", function () {
                expect(contracts.Constants.FINALIZED_ROOT_INDEX).to.equal(105);
            });

            it("FINALIZED_ROOT_DEPTH should be 6", function () {
                expect(contracts.Constants.FINALIZED_ROOT_DEPTH).to.equal(6);
            });

            it("MIN_SYNC_COMMITTEE_PARTICIPANTS should be 1", function () {
                expect(contracts.Constants.MIN_SYNC_COMMITTEE_PARTICIPANTS).to.equal(1);
            });

            it("EMPTY_BEACON_HEADER should be empty and should have correctly sized properties", function () {
                expect(JSON.stringify(contracts.Constants.EMPTY_BEACON_HEADER)).to.equal(JSON.stringify(EMPTY_BEACON_HEADER));
            });

            it("EMPTY_SYNC_COMMITTEE should be empty and should have correctly sized properties", function () {
                expect(JSON.stringify(contracts.Constants.EMPTY_SYNC_COMMITTEE)).to.equal(JSON.stringify(EMPTY_SYNC_COMMITTEE));
            });

            it("EMPTY_LIGHT_CLIENT_UPDATE should be empty and should have correctly sized properties", function () {
                expect(JSON.stringify(contracts.Constants.EMPTY_LIGHT_CLIENT_UPDATE)).to.equal(JSON.stringify(EMPTY_LIGHT_CLIENT_UPDATE));
            });
        });
    });

    describe("HELPERS", function () {
        before(function () {
            contracts.Helpers = new HelpersContract();
        });

        describe("pow", function () {
            it("should make correct calculations", function () {
                expect(contracts.Helpers.pow(2, 2)).to.equal(4);
                expect(contracts.Helpers.pow(2, 3)).to.equal(8);
                expect(contracts.Helpers.pow(3, 2)).to.equal(9);
                expect(contracts.Helpers.pow(3, 3)).to.equal(27);

                expect(contracts.Helpers.pow(0, 2)).to.equal(0);
                expect(contracts.Helpers.pow(2, 0)).to.equal(1);

                expect(contracts.Helpers.pow(7, 7)).to.equal(7 ** 7);
            });

            it("0**0 should equal 1", function () {
                expect(contracts.Helpers.pow(0, 0)).to.equal(1);
            });
        });

        describe("getElementInUintArrayAt", function () {
            it("should NOT throw if index is greater or equal than array size", function () {
                expect(() => contracts.Helpers.getElementInUintArrayAt(1, [])).to.not.throw();
                expect(contracts.Helpers.getElementInUintArrayAt(1, [])).to.equal(0);
            });

            it("should return the right element by index", function () {
                expect(contracts.Helpers.getElementInUintArrayAt(3, [1, 2, 3, 4])).to.equal(4);
                expect(contracts.Helpers.getElementInUintArrayAt(2, [1, 2, 3, 4])).to.equal(3);
                expect(contracts.Helpers.getElementInUintArrayAt(1, [1, 2, 3, 4])).to.equal(2);
                expect(contracts.Helpers.getElementInUintArrayAt(0, [1, 2, 3, 4])).to.equal(1);

                expect(contracts.Helpers.getElementInUintArrayAt(4, [1, 1, 1, 1, 7, 1, 1, 1, 1])).to.equal(7);
            });
        });

        describe("getElementInBytesArrayAt", function () {
            it("should NOT throw if index is greater or equal than array size", function () {
                expect(() => contracts.Helpers.getElementInBytesArrayAt(1, [])).to.not.throw();
                expect(Uint8ArrayToHexString(contracts.Helpers.getElementInBytesArrayAt(1, []))).to.equal(Uint8ArrayToHexString(EMPTY_BYTES32));
            });

            it("should return the right element by index", function () {
                expect(contracts.Helpers.getElementInUintArrayAt(3, ["0x01", "0x02", "0x03", "0x04"])).to.equal("0x04");
                expect(contracts.Helpers.getElementInUintArrayAt(2, ["0x01", "0x02", "0x03", "0x04"])).to.equal("0x03");
                expect(contracts.Helpers.getElementInUintArrayAt(1, ["0x01", "0x02", "0x03", "0x04"])).to.equal("0x02");
                expect(contracts.Helpers.getElementInUintArrayAt(0, ["0x01", "0x02", "0x03", "0x04"])).to.equal("0x01");

                expect(contracts.Helpers.getElementInUintArrayAt(4, ["0x01", "0x01", "0x01", "0x01", "0x07", "0x01", "0x01", "0x01", "0x01"])).to.equal("0x07");
            });
        });

        describe("setElementInBytesArrayAt", function () {
            it("should throw if index is greater or equal than array size", function () {
                expect(() => contracts.Helpers.setElementInBytesArrayAt(1, [])).to.throw();
            });

            it("should return the right element by index", function () {
                expect(JSON.stringify(contracts.Helpers.setElementInBytesArrayAt(3, ["0x01", "0x02", "0x03", "0x04"], "0x00")))
                    .to.equal(JSON.stringify(["0x01", "0x02", "0x03", "0x00"]));
                expect(JSON.stringify(contracts.Helpers.setElementInBytesArrayAt(2, ["0x01", "0x02", "0x00", "0x04"], "0x00")))
                    .to.equal(JSON.stringify(["0x01", "0x02", "0x00", "0x04"]));
                expect(JSON.stringify(contracts.Helpers.setElementInBytesArrayAt(1, ["0x01", "0x00", "0x03", "0x04"], "0x00")))
                    .to.equal(JSON.stringify(["0x01", "0x00", "0x03", "0x04"]));
                expect(JSON.stringify(contracts.Helpers.setElementInBytesArrayAt(0, ["0x00", "0x02", "0x03", "0x04"], "0x00")))
                    .to.equal(JSON.stringify(["0x00", "0x02", "0x03", "0x04"]));

                expect(JSON.stringify(contracts.Helpers.setElementInBytesArrayAt(4, ["0x01", "0x01", "0x01", "0x01", "0x07", "0x01", "0x01", "0x01", "0x01"], "0x01")))
                    .to.equal(JSON.stringify(["0x01", "0x01", "0x01", "0x01", "0x01", "0x01", "0x01", "0x01", "0x01"]));
            });
        });
    });

    describe("UTILS", function () {
        before(function () {
            contracts.Utils = new UtilsContract();
        });

        describe("reverse64", function () {
            it("should return 0 if input is 0", function () {
                expect(contracts.Utils.reverse64(0)).to.equal(0);
            });

            it("should calculate correct reversed value in 64 bits size of 1", function () {
                expect(contracts.Utils.reverse64(1)).to.equal(parseInt(72057594037927936n));
                expect(contracts.Utils.reverse64(parseInt(72057594037927936n))).to.equal(1);
            });

            it("should calculate correct reversed value in 64 bits size of 2", function () {
                expect(contracts.Utils.reverse64(2)).to.equal(parseInt(144115188075855872n));
                expect(contracts.Utils.reverse64(parseInt(144115188075855872n))).to.equal(2);
            });
        });

        describe("to_little_endian_64", function () {
            it("should return the reverse64 value of input as bytes", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.to_little_endian_64(0)))
                    .to.equal(Uint8ArrayToHexString([...Array(7).fill(0), ...[0]]));
                expect(Uint8ArrayToHexString(contracts.Utils.to_little_endian_64(parseInt(72057594037927936n))))
                    .to.equal(Uint8ArrayToHexString([...Array(7).fill(0), ...[1]]));
                expect(Uint8ArrayToHexString(contracts.Utils.to_little_endian_64(parseInt(144115188075855872n))))
                    .to.equal(Uint8ArrayToHexString([...Array(7).fill(0), ...[2]]));
            });
        });

        describe("compute_epoch_at_slot", function () {
            it("should return floored integer dividing the input by the SLOTS_PER_EPOCH value (32 slots/epoch)", function () {
                expect(contracts.Utils.compute_epoch_at_slot(0)).to.equal(Math.floor(0 / contracts.Utils.SLOTS_PER_EPOCH));
                expect(contracts.Utils.compute_epoch_at_slot(31)).to.equal(Math.floor(31 / contracts.Utils.SLOTS_PER_EPOCH));
                expect(contracts.Utils.compute_epoch_at_slot(32)).to.equal(Math.floor(32 / contracts.Utils.SLOTS_PER_EPOCH));
                expect(contracts.Utils.compute_epoch_at_slot(33)).to.equal(Math.floor(33 / contracts.Utils.SLOTS_PER_EPOCH));
            });
        });

        describe("get_power_of_two_ceil", function () {
            it("should return correct ceiled integer", function () {
                expect(contracts.Utils.get_power_of_two_ceil(0)).to.equal(1);
                expect(contracts.Utils.get_power_of_two_ceil(1)).to.equal(1);
                expect(contracts.Utils.get_power_of_two_ceil(2)).to.equal(2);
                expect(contracts.Utils.get_power_of_two_ceil(3)).to.equal(4);
                expect(contracts.Utils.get_power_of_two_ceil(4)).to.equal(4);
                expect(contracts.Utils.get_power_of_two_ceil(5)).to.equal(8);
                expect(contracts.Utils.get_power_of_two_ceil(9)).to.equal(16);
                expect(contracts.Utils.get_power_of_two_ceil(17)).to.equal(32);
                expect(contracts.Utils.get_power_of_two_ceil(1029)).to.equal(2048);
            });
        });

        describe("merkle_root", function () {
            it("should return empty hash if array is empty", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.merkle_root([])))
                    .to.equal("0x".concat(Array(32 * 2 + 1).join("0")));
            });

            it("should return hash of single element if array.length equals 1", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.merkle_root([EMPTY_BYTES32])))
                    .to.equal("0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925");
            });

            it("should return hash of both elements concatenated if array.length equals 2", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.merkle_root([EMPTY_BYTES32, EMPTY_BYTES32])))
                    .to.equal("0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b");
            });

            it("should return correct merkle hashes when more than 2 elements (4 * EMPTY_BYTES32)", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.merkle_root([EMPTY_BYTES32, EMPTY_BYTES32, EMPTY_BYTES32, EMPTY_BYTES32])))
                    .to.equal("0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71");
            });

            it("should return correct merkle hashes when more than 2 elements (random hashes - precalculated root)", function () {
                expect(Uint8ArrayToHexString(contracts.Utils.merkle_root(RANDOM_BYTES)))
                    .to.equal(Uint8ArrayToHexString(RANDOM_BYTES_ROOT));
            });
        });

        describe("is_valid_merkle_branch", function () {
            it("should verify a correct input (4 * EMPTY_BYTES32)", function () {
                const leaf = digest(EMPTY_BYTES32);
                const branch = [
                    digest(EMPTY_BYTES32),
                    digest([...digest(EMPTY_BYTES32), ...digest(EMPTY_BYTES32)])
                ];
                const depth = 2;
                const index = 1;
                const root = contracts.Utils.merkle_root([digest(EMPTY_BYTES32), digest(EMPTY_BYTES32), digest(EMPTY_BYTES32), digest(EMPTY_BYTES32)]);

                expect(contracts.Utils.is_valid_merkle_branch(
                    leaf,
                    branch,
                    depth,
                    index,
                    root
                )).to.be.true;
            });

            it("should verify a correct input (random hashes - precalculated root)", function () {
                const leaf = digest(RANDOM_BYTES[1]);
                const branch = [
                    digest(RANDOM_BYTES[0]),
                    digest([...digest(RANDOM_BYTES[2]), ...digest(RANDOM_BYTES[3])])
                ];
                const depth = 2;
                const index = 1;
                const root = contracts.Utils.merkle_root(RANDOM_BYTES.map(digest));

                expect(contracts.Utils.is_valid_merkle_branch(
                    leaf,
                    branch,
                    depth,
                    index,
                    root
                )).to.be.true;
            });
        });

        describe("hash_tree_root", function () {
            describe("fork_data", function () {
                it("should return correct fork data hash (empty fork data)", function () {
                    expect(Uint8ArrayToHexString(contracts.Utils.hash_tree_root__fork_data({
                        current_version: EMPTY_BYTES4,
                        genesis_validators_root: EMPTY_BYTES32,
                    }))).to.equal("0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b");
                });
            });

            describe("signing_data", function () {
                it("should return correct signing data hash (empty signing data)", function () {
                    expect(Uint8ArrayToHexString(contracts.Utils.hash_tree_root__signing_data({
                        object_root: EMPTY_BYTES32,
                        domain: EMPTY_BYTES32,
                    }))).to.equal("0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b");
                });
            });

            describe("block_header", function () {
                it("should return correct block header hash (EMPTY_BEACON_HEADER)", function () {
                    expect(Uint8ArrayToHexString(contracts.Utils.hash_tree_root__block_header(EMPTY_BEACON_HEADER)))
                        .to.equal("0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c");
                });

                it("should return correct block header hash (normal header)", function () {
                    const header = BLC_UPDATES[0].attested_header;

                    expect(Uint8ArrayToHexString(contracts.Utils.hash_tree_root__block_header(header)))
                        .to.equal("0xe367fd4fce2eaa248d8d970ff2836a032b7c1dcea02b6f1bce831e13443e6f1e");
                });
            });

            describe("sync_committee", function () {
                it("should return correct sync committee hash", function () {
                    const sync_committee = BLC_UPDATES[0].next_sync_committee;

                    expect(Uint8ArrayToHexString(contracts.Utils.hash_tree_root__sync_committee(sync_committee)))
                        .to.equal("0x5b40b6aefbdbcffec9c5b7f977c4b4d009a2fdfd1d239f36731d604cfa732970");
                });
            });
        });

        describe("compute", function () {
            describe("fork_data_root", function () {
                it("should return correct fork data hash (empty fork data, same as hash_tree_root)", function () {
                    expect(Uint8ArrayToHexString(contracts.Utils.compute_fork_data_root(EMPTY_BYTES4, EMPTY_BYTES32)))
                        .to.equal("0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b");

                    expect(Uint8ArrayToHexString(contracts.Utils.compute_fork_data_root(EMPTY_BYTES4, EMPTY_BYTES32)))
                        .to.equal(
                            Uint8ArrayToHexString(contracts.Utils.hash_tree_root__fork_data({
                                current_version: EMPTY_BYTES4,
                                genesis_validators_root: EMPTY_BYTES32,
                            }))
                        );
                });
            });

            describe("domain", function () {
                it("should properly hash, calculate and return domain", function () {
                    const domain_type = contracts.Utils.DOMAIN_SYNC_COMMITTEE;
                    const fork_version = EMPTY_BYTES4;
                    const genesis_validators_root = EMPTY_BYTES32;

                    expect(Uint8ArrayToHexString(contracts.Utils.compute_domain(domain_type, fork_version, genesis_validators_root)))
                        .to.equal("0x07000000f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a9");
                });
            });

            describe("signing_root", function () {
                it("should return correct signing data hash (empty fork data, same as hash_tree_root)", function () {
                    const domain_type = contracts.Utils.DOMAIN_SYNC_COMMITTEE;
                    const fork_version = EMPTY_BYTES4;
                    const genesis_validators_root = EMPTY_BYTES32;

                    const domain = contracts.Utils.compute_domain(domain_type, fork_version, genesis_validators_root);
                    expect(Uint8ArrayToHexString(contracts.Utils.compute_signing_root(EMPTY_BEACON_HEADER, domain)))
                        .to.equal("0xcad4bd07350cdacd98296af6e323e08c185709bc4a0eb44182e47d1f56f7fb68");
                });
            });
        });
    });

    describe("BEACON_LIGHT_CLIENT", function () {
        before(async function () {
            contracts.blc = new BeaconLightClientContract({
                snapshot: BLC_SNAPSHOT,
                valid_updates: []
            }, EMPTY_BYTES32);
        });

        it("process_light_client_update should pass all updates", function () {
            let counter = 291;
            for (let update of BLC_UPDATES) {
                console.log(` >>> Processing update at period ${counter++}...`);
                contracts.blc.process_light_client_update(update, update.attested_header.slot);
            }
        });
    });
});