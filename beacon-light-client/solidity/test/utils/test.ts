import { expand_message_xmd, stringToBytes, htfDefaults, hash_to_field } from "./bls";
import { map_to_curve_simple_swu_9mod16 } from "../../../../vendor/circom-pairing/test/math"
import { PointG2 } from '@noble/bls12-381';
import { formatHex } from '@dendreth/utils/ts-utils/bls';

function bigintToBytes(value: bigint): Uint8Array {
    // Determine the required number of bytes to represent the bigint
    const byteLength = Math.ceil(value.toString(16).length / 2);

    // Initialize a Uint8Array to hold the bytes
    const byteArray = new Uint8Array(byteLength);

    // Convert the bigint to bytes
    for (let i = 0; i < byteLength; i++) {
        // Get the least significant byte and store it in the array
        byteArray[byteLength - i - 1] = Number(value & BigInt(0xFF));
        // Shift the value to the right by 8 bits (1 byte)
        value >>= BigInt(8);
    }

    return byteArray;
}

function bigintTo12Limbs(value: bigint): bigint[] {
    const numLimbs = 12; // Number of limbs
    const limbSize = 64; // Each limb size in bits

    // Create an array to hold the limbs
    const limbs = new Array<bigint>(numLimbs);

    // Loop through each limb and extract 64 bits at a time
    for (let i = 0; i < numLimbs; i++) {
        // Use a mask to extract the least significant 64 bits
        const mask = (BigInt(1) << BigInt(limbSize)) - BigInt(1);
        limbs[i] = value & mask;
        // Shift the value to the right by 64 bits for the next limb
        value >>= BigInt(limbSize);
    }

    return limbs;
}

function uint8ArrayToHexString(arr: Uint8Array): string {
    return Array.from(arr)
        .map(byte => byte.toString(16).padStart(2, '0'))
        .join('');
}

(async () => {
    let msg = new Uint8Array([9636, 8499, 980, 3289, 2380, 4091, 4494, 7841, 8175, 1645, 9486, 6069, 8507, 739, 4264, 209, 1174, 7352, 1824, 5981, 3557, 8703, 368, 9610, 6902, 3]);
    const DST = stringToBytes(htfDefaults.DST);

    let hash_to_field_result = await hash_to_field(msg, 2);

    // let hash_to_curve_test_res: PointG2 = await PointG2.hashToCurve(msg);
    let map_to_curve: PointG2 = map_to_curve_simple_swu_9mod16(hash_to_field_result[0]);

    let hash_to_curve_test_res: PointG2 = await PointG2.hashToCurve(
        formatHex(uint8ArrayToHexString(msg)),
    );

    console.log('hash_to_field_result is: ', hash_to_field_result);
    console.log('####################################################');
    console.log('x is: ', hash_to_curve_test_res.x);
    console.log('y is: ', hash_to_curve_test_res.y);


    // let a = 164432780807461518526223636504241229349588394649409730072519387299403412015098917482545551400313990282635303577913n;

    // for (let i = 1n; i <= 12n; i++) {
    //     console.log((a % (2n ** 32n)));
    //     a = a / (2n ** 32n);
    // }
})();