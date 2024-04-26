import { expand_message_xmd, stringToBytes, htfDefaults, hash_to_field } from "./bls";

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

(async () => {
    let msg = new Uint8Array([0, 0]);
    const DST = stringToBytes(htfDefaults.DST);

    // let result = await expand_message_xmd(msg, DST, 2);
    let hash_to_field_result = await hash_to_field(msg, 2);

    let hash_to_field_res_0_0 = hash_to_field_result[0][0];

    // console.log(result);
    console.log('hash_to_field_result is: ', hash_to_field_result);
    console.log('hash_to_field_res_0_0 is: ', hash_to_field_res_0_0);
    //console.log('bytes of hash_to_field_res_0_0 are: ', bigintToBytes(hash_to_field_res_0_0));
    //console.log('bigintTo12Limbs of hash_to_field_res_0_0 are: ', bigintTo12Limbs(hash_to_field_res_0_0));
})();