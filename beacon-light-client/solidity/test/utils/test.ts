import { expand_message_xmd, stringToBytes, htfDefaults, hash_to_field } from "./bls";
import { Fp2, isogenyMapG2, map_to_curve_simple_swu_9mod16 } from "../../../../vendor/circom-pairing/test/math"
import { PointG2 } from "../../../../vendor/circom-pairing/test/index"
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

type Fp2_4 = [Fp2, Fp2, Fp2, Fp2];

const xnum = [
    [
        0x171d6541fa38ccfaed6dea691f5fb614cb14b4e7f4e810aa22d6108f142b85757098e38d0f671c7188e2aaaaaaaa5ed1n,
        0x0n,
    ],
    [
        0x11560bf17baa99bc32126fced787c88f984f87adf7ae0c7f9a208c6b4f20a4181472aaa9cb8d555526a9ffffffffc71en,
        0x8ab05f8bdd54cde190937e76bc3e447cc27c3d6fbd7063fcd104635a790520c0a395554e5c6aaaa9354ffffffffe38dn,
    ],
    [
        0x0n,
        0x11560bf17baa99bc32126fced787c88f984f87adf7ae0c7f9a208c6b4f20a4181472aaa9cb8d555526a9ffffffffc71an,
    ],
    [
        0x5c759507e8e333ebb5b7a9a47d7ed8532c52d39fd3a042a88b58423c50ae15d5c2638e343d9c71c6238aaaaaaaa97d6n,
        0x5c759507e8e333ebb5b7a9a47d7ed8532c52d39fd3a042a88b58423c50ae15d5c2638e343d9c71c6238aaaaaaaa97d6n,
    ],
].map((pair) => Fp2.fromBigTuple(pair)) as Fp2_4;
const xden = [
    [0x0n, 0x0n],
    [0x1n, 0x0n],
    [
        0xcn,
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaa9fn,
    ],
    [
        0x0n,
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaa63n,
    ],
].map((pair) => Fp2.fromBigTuple(pair)) as Fp2_4;
const ynum = [
    [
        0x124c9ad43b6cf79bfbf7043de3811ad0761b0f37a1e26286b0e977c69aa274524e79097a56dc4bd9e1b371c71c718b10n,
        0x0n,
    ],
    [
        0x11560bf17baa99bc32126fced787c88f984f87adf7ae0c7f9a208c6b4f20a4181472aaa9cb8d555526a9ffffffffc71cn,
        0x8ab05f8bdd54cde190937e76bc3e447cc27c3d6fbd7063fcd104635a790520c0a395554e5c6aaaa9354ffffffffe38fn,
    ],
    [
        0x0n,
        0x5c759507e8e333ebb5b7a9a47d7ed8532c52d39fd3a042a88b58423c50ae15d5c2638e343d9c71c6238aaaaaaaa97ben,
    ],
    [
        0x1530477c7ab4113b59a4c18b076d11930f7da5d4a07f649bf54439d87d27e500fc8c25ebf8c92f6812cfc71c71c6d706n,
        0x1530477c7ab4113b59a4c18b076d11930f7da5d4a07f649bf54439d87d27e500fc8c25ebf8c92f6812cfc71c71c6d706n,
    ],
].map((pair) => Fp2.fromBigTuple(pair)) as Fp2_4;
const yden = [
    [0x1n, 0x0n],
    [
        0x12n,
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaa99n,
    ],
    [
        0x0n,
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffa9d3n,
    ],
    [
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffa8fbn,
        0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffa8fbn,
    ],
].map((pair) => Fp2.fromBigTuple(pair)) as Fp2_4;
const ISOGENY_COEFFICIENTS_G2: [Fp2_4, Fp2_4, Fp2_4, Fp2_4] = [xnum, xden, ynum, yden];


function hexToBytes(hex: string): Uint8Array {
    if (typeof hex !== "string") {
        throw new TypeError("hexToBytes: expected string, got " + typeof hex);
    }
    if (hex.length % 2)
        throw new Error("hexToBytes: received invalid unpadded hex");
    const array = new Uint8Array(hex.length / 2);
    for (let i = 0; i < array.length; i++) {
        const j = i * 2;
        const hexByte = hex.slice(j, j + 2);
        if (hexByte.length !== 2) throw new Error("Invalid byte sequence");
        const byte = Number.parseInt(hexByte, 16);
        if (Number.isNaN(byte) || byte < 0)
            throw new Error("Invalid byte sequence");
        array[i] = byte;
    }
    return array;
}


// // 3-isogeny map from E' to E
// // https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-11#appendix-E.3
// function nobleIsogenyMap<T extends Field<T>>(COEFF: [T[], T[], T[], T[]], x: T, y: T): [T, T] {
//     const [xNum, xDen, yNum, yDen] = COEFF.map((val) =>
//         val.reduce((acc, i) => acc.multiply(x).add(i))
//     );
//     x = xNum.div(xDen); // xNum / xDen
//     y = y.multiply(yNum.div(yDen)); // y * (yNum / yDev)
//     return [x, y];
// }

function ensureBytes(hex: string | Uint8Array): Uint8Array {
    // Uint8Array.from() instead of hash.slice() because node.js Buffer
    // is instance of Uint8Array, and its slice() creates **mutable** copy
    return hex instanceof Uint8Array ? Uint8Array.from(hex) : hexToBytes(hex);
}
type Hex = Uint8Array | string;

// Encodes byte string to elliptic curve
// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-11#section-3
// async function testHashToCurve(msg: Hex) {
//     msg = ensureBytes(msg);
//     const u = await hash_to_field(msg, 2);
//     // console.log(`hash_to_curve(msg}) u0=${new Fp2(u[0])} u1=${new Fp2(u[1])}`);
//     console.log("map_to_curve_simple_swu_9mod16", map_to_curve_simple_swu_9mod16(u[0]));
//     const Q0 = new PointG2(
//         ...isogenyMapG2(map_to_curve_simple_swu_9mod16(u[0]))
//     );
//     const Q1 = new PointG2(
//         ...isogenyMapG2(map_to_curve_simple_swu_9mod16(u[1]))
//     );
//     // const R = Q0.add(Q1);

//     return u;
// }

(async () => {
    let msg = new Uint8Array([9636, 8499, 980, 3289, 2380, 4091, 4494, 7841, 8175, 1645, 9486, 6069, 8507, 739, 4264, 209, 1174, 7352, 1824, 5981, 3557, 8703, 368, 9610, 6902, 3]);
    const DST = stringToBytes(htfDefaults.DST);

    let hash_to_field_result = await hash_to_field(msg, 2);
    // let map_to_curve: PointG2 = map_to_curve_simple_swu_9mod16(hash_to_field_result[0]);
    // let iso_map_r = nobleIsogenyMap(ISOGENY_COEFFICIENTS_G2, map_to_curve[0], map_to_curve[1]);
    // let clear_cof_g2_r = clearCofactor(iso_map_r);

    let hash_to_curve_test_res: PointG2 = await PointG2.hashToCurve(
        formatHex(uint8ArrayToHexString(msg)),
    );

    // let without_cofactor_hash2curve = await testHashToCurve(msg);

    // console.log('hash_to_field_result is: ', hash_to_field_result);
    console.log('hash_to_curve_test_res.x is: ', hash_to_curve_test_res.x);
    console.log('hash_to_curve_test_res.y is: ', hash_to_curve_test_res.y);
    console.log('####################################################');
    // console.log('without_cofactor_hash2curve is: ', without_cofactor_hash2curve);


    // let a = 164432780807461518526223636504241229349588394649409730072519387299403412015098917482545551400313990282635303577913n;

    // for (let i = 1n; i <= 12n; i++) {
    //     console.log((a % (2n ** 32n)));
    //     a = a / (2n ** 32n);
    // }
})();