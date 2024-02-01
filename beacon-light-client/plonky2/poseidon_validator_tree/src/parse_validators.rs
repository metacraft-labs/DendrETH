use std::fs::File;
use anyhow::Error;
use serde_json::Value;
use std::io::Read;
use num_bigint::BigUint;
use hex;

use crate::objects::{Validator, ValidatorDataInput};

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

