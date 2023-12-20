use serde_json::{Value, Error};
use serde::Deserialize;
use serde_with::serde_as;
use std::any;
use std::fs::File;
use std::io::{Read, Error as IOError};

use plonky2::hash::hash_types::RichField;
use plonky2x::{
    backend::circuit::{Circuit, DefaultParameters},
    prelude::{bytes32,CircuitVariable,ArrayVariable, BoolVariable, CircuitBuilder, Field, PlonkParameters, Variable, U256Variable}, 
    frontend::{eth::{beacon::vars::BeaconValidatorVariable, vars::BLSPubkeyVariable},
    // frontend::{eth::vars::BLSPubkeyVariable,
        vars::{Bytes32Variable}, uint::uint64::U64Variable, hash::poseidon::poseidon256::PoseidonHashOutVariable},
};

const VALIDATORS_PER_COMMITTEE: usize = 412; // 2048
const PLACEHOLDER: usize = 11;



// #[derive(Debug, Clone, Copy)]
// pub struct BeaconValidatorVariable {
//     pub pubkey: BLSPubkeyVariable,
//     pub withdrawal_credentials: Bytes32Variable,
//     pub effective_balance: U256Variable,
//     pub slashed: BoolVariable,
//     pub activation_eligibility_epoch: U256Variable,
//     pub activation_epoch: U256Variable,
//     pub exit_epoch: U256Variable,
//     pub withdrawable_epoch: U256Variable,
// }

// /// The beacon validator struct according to the consensus spec.
// /// Reference: https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#validator
// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde_as]
// pub struct BeaconValidator {
//     pub pubkey: String,
//     pub withdrawal_credentials: String,
//     pub effective_balance: u64,
//     pub slashed: bool,
//     pub activation_eligibility_epoch: String,
//     pub activation_epoch: String,
//     pub exit_epoch: String,
//     pub withdrawable_epoch: String,
// }

// impl CircuitVariable for BeaconValidatorVariable {
//     type ValueType<F: RichField> = BeaconValidator;

//     fn init_unsafe<L: PlonkParameters<D>, const D: usize>(
//         builder: &mut CircuitBuilder<L, D>,
//     ) -> Self {
//         Self {
//             pubkey: BLSPubkeyVariable::init_unsafe(builder),
//             withdrawal_credentials: Bytes32Variable::init_unsafe(builder),
//             effective_balance: U256Variable::init_unsafe(builder),
//             slashed: BoolVariable::init_unsafe(builder),
//             activation_eligibility_epoch: U256Variable::init_unsafe(builder),
//             activation_epoch: U256Variable::init_unsafe(builder),
//             exit_epoch: U256Variable::init_unsafe(builder),
//             withdrawable_epoch: U256Variable::init_unsafe(builder),
//         }
//     }

//     fn nb_elements() -> usize {
//         let pubkey = BLSPubkeyVariable::nb_elements();
//         let withdrawal_credentials = Bytes32Variable::nb_elements();
//         let effective_balance = U256Variable::nb_elements();
//         let slashed = BoolVariable::nb_elements();
//         let activation_eligibility_epoch = U256Variable::nb_elements();
//         let activation_epoch = U256Variable::nb_elements();
//         let exit_epoch = U256Variable::nb_elements();
//         let withdrawable_epoch = U256Variable::nb_elements();
//         pubkey
//             + withdrawal_credentials
//             + effective_balance
//             + slashed
//             + activation_eligibility_epoch
//             + activation_epoch
//             + exit_epoch
//             + withdrawable_epoch
//     }

    // fn elements<F: RichField>(value: Self::ValueType<F>) -> Vec<F> {
    //     let pubkey = BLSPubkeyVariable::elements(bytes!(value.pubkey));
    //     let withdrawal_credentials =
    //         Bytes32Variable::elements(bytes32!(value.withdrawal_credentials));
    //     let effective_balance = U256Variable::elements(value.effective_balance.into());
    //     let slashed = BoolVariable::elements(value.slashed);
    //     let activation_eligibility_epoch = U256Variable::elements(
    //         value
    //             .activation_eligibility_epoch
    //             .parse::<u64>()
    //             .unwrap()
    //             .into(),
    //     );
    //     let activation_epoch =
    //         U256Variable::elements(value.activation_epoch.parse::<u64>().unwrap().into());
    //     let exit_epoch = U256Variable::elements(value.exit_epoch.parse::<u64>().unwrap().into());
    //     let withdrawable_epoch =
    //         U256Variable::elements(value.withdrawable_epoch.parse::<u64>().unwrap().into());
    //     pubkey
    //         .into_iter()
    //         .chain(withdrawal_credentials)
    //         .chain(effective_balance)
    //         .chain(slashed)
    //         .chain(activation_eligibility_epoch)
    //         .chain(activation_epoch)
    //         .chain(exit_epoch)
    //         .chain(withdrawable_epoch)
    //         .collect()
    // }

    // fn from_elements<F: RichField>(elements: &[F]) -> Self::ValueType<F> {
    //     let pubkey = BLSPubkeyVariable::from_elements(&elements[0..384]);
    //     let withdrawal_credentials = Bytes32Variable::from_elements(&elements[384..640]);
    //     let effective_balance = U256Variable::from_elements(&elements[640..648]);
    //     let slashed = BoolVariable::from_elements(&elements[648..649]);
    //     let activation_eligibility_epoch = U256Variable::from_elements(&elements[649..657]);
    //     let activation_epoch = U256Variable::from_elements(&elements[657..665]);
    //     let exit_epoch = U256Variable::from_elements(&elements[665..673]);
    //     let withdrawable_epoch = U256Variable::from_elements(&elements[673..681]);
    //     BeaconValidator {
    //         pubkey: hex!(pubkey),
    //         withdrawal_credentials: hex!(withdrawal_credentials),
    //         effective_balance: effective_balance.as_u64(),
    //         slashed,
    //         activation_eligibility_epoch: activation_eligibility_epoch.as_u64().to_string(),
    //         activation_epoch: activation_epoch.as_u64().to_string(),
    //         exit_epoch: exit_epoch.as_u64().to_string(),
    //         withdrawable_epoch: withdrawable_epoch.as_u64().to_string(),
    //     }
    // }

//     fn variables(&self) -> Vec<Variable> {
//         let mut vars = Vec::new();
//         vars.extend(self.pubkey.variables());
//         vars.extend(self.withdrawal_credentials.variables());
//         vars.extend(self.effective_balance.variables());
//         vars.extend(self.slashed.variables());
//         vars.extend(self.activation_eligibility_epoch.variables());
//         vars.extend(self.activation_epoch.variables());
//         vars.extend(self.exit_epoch.variables());
//         vars.extend(self.withdrawable_epoch.variables());
//         vars
//     }

//     fn from_variables_unsafe(variables: &[Variable]) -> Self {
//         let pubkey = BLSPubkeyVariable::from_variables_unsafe(&variables[0..384]);
//         let withdrawal_credentials = Bytes32Variable::from_variables_unsafe(&variables[384..640]);
//         let effective_balance = U256Variable::from_variables_unsafe(&variables[640..648]);
//         let slashed = BoolVariable::from_variables_unsafe(&variables[648..649]);
//         let activation_eligibility_epoch =
//             U256Variable::from_variables_unsafe(&variables[649..657]);
//         let activation_epoch = U256Variable::from_variables_unsafe(&variables[657..665]);
//         let exit_epoch = U256Variable::from_variables_unsafe(&variables[665..673]);
//         let withdrawable_epoch = U256Variable::from_variables_unsafe(&variables[673..681]);
//         Self {
//             pubkey,
//             withdrawal_credentials,
//             effective_balance,
//             slashed,
//             activation_eligibility_epoch,
//             activation_epoch,
//             exit_epoch,
//             withdrawable_epoch,
//         }
//     }

//     fn assert_is_valid<L: PlonkParameters<D>, const D: usize>(
//         &self,
//         builder: &mut CircuitBuilder<L, D>,
//     ) {
//         self.pubkey.assert_is_valid(builder);
//         self.withdrawal_credentials.assert_is_valid(builder);
//         self.effective_balance.assert_is_valid(builder);
//         self.slashed.assert_is_valid(builder);
//         self.activation_eligibility_epoch.assert_is_valid(builder);
//         self.activation_epoch.assert_is_valid(builder);
//         self.exit_epoch.assert_is_valid(builder);
//         self.withdrawable_epoch.assert_is_valid(builder);
//     }
// }


pub fn prove_verify_attestation_data(attestations: &Vec<Value>, mut builder: CircuitBuilder<DefaultParameters, 2>) {
    // For each attestation run VerifyAttestationData (TODO: Missing validator_list_proof from original object)
    for attestation in attestations {
        // Parse Data and register as inputs for circuit

        //data
        if let Some(data) = attestation.get("data") {
            if let Some(slot) = data.get("slot") {
                println!("AttestationData Slot: {}", slot);
                builder.read::<U256Variable>();

            }
            if let Some(index) = data.get("index") {
                println!("AttestationData Index: {}", index);
            }
            if let Some(beacon_block_root) = data.get("beacon_block_root") {
                println!("AttestationData Beacon Block Root: {}", beacon_block_root);
            }
            if let Some(source) = data.get("source") {
                if let Some(epoch) = source.get("epoch") {
                    println!("==>Epoch: {}", epoch);
                }
                if let Some(root) = source.get("root") {
                    println!("==>Root: {}", root);
                }
            }
            if let Some(source) = data.get("target") {
                if let Some(epoch) = source.get("epoch") {
                    println!("==>Epoch: {}", epoch);
                }
                if let Some(root) = source.get("root") {
                    println!("==>Root: {}", root);
                }
            }
        }

        //signature
        if let Some(signature) = attestation.get("signature").and_then(Value::as_str) {
            println!("Signature: {}", signature);
        }
        //fork
        if let Some(fork) = attestation.get("fork") {
            if let Some(previous_version) = fork.get("previous_version") {
                println!("Fork Previous Version: {}", previous_version);
            }
            if let Some(current_version) = fork.get("current_version") {
                println!("Fork Current Version: {}", current_version);
            }
            if let Some(epoch) = fork.get("epoch") {
                println!("Fork Epoch: {}", epoch);
            }
        }

        //genesis_validators_root
        if let Some(state_root) = attestation.get("state_root").and_then(Value::as_str) {
            println!("State Root: {}", state_root);
        }

        //state_root_proof
        if let Some(state_root_proof) = attestation.get("state_root_proof")
            .and_then(Value::as_array) {
                println!("State_root_proof: ");
                for branch in state_root_proof {
                    println!("==> {}", branch);
                }
            }

        //validators_root
        if let Some(validators_root) = attestation.get("validators_root").and_then(Value::as_str) {
            println!("Validators Root: {}", validators_root)
        }
        
        //validators_root_proof
        if let Some(validators_root_proof) = attestation.get("validators_root_proof")
            .and_then(Value::as_array) {
                println!("Validators_root_proof: ");
                for branch in validators_root_proof {
                    println!("==> {}", branch);
                }
            }
        }
}

fn print_json_value(value: &Value, indent: usize) {
    match value {
        Value::Null => println!("null"),
        Value::Bool(b) => println!("{}", b),
        Value::Number(num) => println!("{}", num),
        Value::String(s) => println!("\"{}\"", s),
        Value::Array(arr) => {
            println!("[");
            for item in arr {
                print!("{}  ", " ".repeat(indent + 2));
                print_json_value(item, indent + 2);
            }
            println!("{}]", " ".repeat(indent));
        }
        Value::Object(obj) => {
            println!("{}", "{");
            for (key, value) in obj {
                print!("{}\"{}\": ", " ".repeat(indent + 2), key);
                print_json_value(value, indent + 2);
            }
            println!("{}}}", " ".repeat(indent));
        }
    }
}
