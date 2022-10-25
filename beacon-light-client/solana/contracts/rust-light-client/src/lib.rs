use std::{convert::TryInto, ops::Neg};

use ark_bn254::{Bn254, Fq, Fq2, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_groth16::PreparedVerifyingKey;
use borsh::{BorshDeserialize, BorshSerialize};
use num_bigint::BigUint;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LightClientAccount {
    pub public_finalized_header_root: [u8; 32],
    pub prev_block_header_root: [u8; 32],
}

pub mod instruction;
use crate::instruction::ProofData;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");

    if instruction_data.len() != 384 {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    let pesho: u8 = 0;
    let proof_data = ProofData::unpack(
        instruction_data
            .try_into()
            .expect("Invalid instruction data"),
    );

    msg!("Array allocated");
    msg!("VK");
    let gosho: u8 = 0;


    msg!("main programm: {:p}", &pesho);
    msg!("main programm: {:p}", &gosho);

    msg!("PVK");

    // if ark_groth16::verify_proof(&proof_data.pvk, &proof_data.proof, &proof_data.pub_input).unwrap()
    // {
    //     msg!("Valid proof for everything is perfect");
    //     return Ok(());
    // } else {
    //     msg!("Valid proof for everything is perfect");
    //     return Err(ProgramError::InvalidInstructionData);
    // }

    Ok(())
}

