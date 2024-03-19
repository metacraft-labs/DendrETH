
use std::fs::File;
use anyhow::Error;
use serde_json::Value;
use std::io::Read;
use num_bigint::BigUint;
use hex;

use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Validator {
    pub trusted: bool,
    pub validator_index: u64,

    pub pubkey: [bool; 384],
    pub withdrawal_credentials: [bool; 256],
    pub effective_balance: BigUint,
    pub slashed: bool,
    pub activation_eligibility_epoch: BigUint,
    pub activation_epoch: BigUint,
    pub exit_epoch: BigUint,
    pub withdrawable_epoch: BigUint,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataInput {
    pub trusted: bool,
    pub validator_index: u64,

    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub effective_balance: u64,
    pub exit_epoch: u64,
    pub pubkey: String,
    pub slashed: bool,
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidatorPoseidonDataOutput {
    pub trusted: bool,
    pub validator_index: u64,

    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub effective_balance: u64,
    pub exit_epoch: u64,
    pub pubkey: String,
    pub slashed: bool,
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
    pub validator_poseidon_hash: HashOut<GoldilocksField>,
    pub validator_poseidon_proof: Vec<HashOut<GoldilocksField>>
}


pub fn read_validator_data(file_path_attestations: &str)  -> Vec<Validator> {
    
    let validators_json = read_json_from_file(file_path_attestations).unwrap();
    let validators_data: Value = serde_json::from_value(validators_json.clone()).unwrap();
    
    let validators: Vec<ValidatorDataInput> = validators_data.get("data")
    .and_then(Value::as_array)
    .map(|array| {
        array
        .iter()
        .flat_map(|validator| {serde_json::from_value(validator.clone())})
        .collect()
    }).unwrap();
    
    let mut parsed_validators: Vec<Validator> = vec![];
    for validator in validators {
        parsed_validators.push(parse_validator(validator));
    }
    
    parsed_validators
}

pub fn read_json_from_file(path_to_file: &str) -> Result<Value, Error> {
    let mut file = File::open(path_to_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(serde_json::from_str(&contents)?)
}

pub fn parse_validator(validator: ValidatorDataInput) -> Validator {
    
    Validator {
        trusted: validator.trusted,
        validator_index: validator.validator_index,

        pubkey: hex_to_binary(validator.pubkey.as_str()).try_into().unwrap(),
        withdrawal_credentials: hex_to_binary(validator.withdrawal_credentials.as_str()).try_into().unwrap(),
        effective_balance: BigUint::from(validator.effective_balance),
        slashed: validator.slashed,
        activation_eligibility_epoch: BigUint::from(validator.activation_eligibility_epoch),
        activation_epoch: BigUint::from(validator.activation_epoch),
        exit_epoch: BigUint::from(validator.exit_epoch),
        withdrawable_epoch: BigUint::from(validator.withdrawable_epoch)
    }

}

fn hex_to_binary(hex: &str) -> Vec<bool> {
    let clean_hex = if hex.starts_with("0x") { &hex[2..] } else { hex };
    let bytes = hex::decode(clean_hex).expect("Invalid hexadecimal number");
    
    let mut result = Vec::new();
    for byte in bytes {
        for i in (0..8).rev() {
            result.push((byte >> i) & 1 == 1);
        }
    }

    result
}

pub fn binary_to_hex(binary: &[bool]) -> String {
    let mut hex_string = String::new();

    // Iterate over the binary vector in chunks of 4 bits
    for chunk in binary.chunks(4) {
        // Convert the 4-bit chunk to a u8 value
        let mut byte = 0;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << (3 - i);
            }
        }

        // Convert the u8 value to a hexadecimal character and append to the string
        hex_string.push_str(&format!("{:X}", byte));
    }

    format!("0x{}", hex_string)
}
