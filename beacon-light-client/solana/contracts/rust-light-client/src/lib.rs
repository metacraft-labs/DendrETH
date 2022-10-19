use std::convert::TryInto;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LightClientAccount {
    pub public_finalized_header_root: [u8; 32],
    pub prev_block_header_root: [u8; 32],
}

pub mod circom;
pub mod instruction;
use crate::{circom::run_verifier, instruction::ProofData};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");

    let accounts_iter = &mut accounts.iter();

    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Light client account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    if instruction_data.len() != 384 {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    let proof_data = ProofData::unpack(
        instruction_data
            .try_into()
            .expect("Invalid instruction data"),
    );

    msg!("Managed to unpack {}", proof_data.proof.a);

    if run_verifier(&proof_data) {
        msg!("Valid proof for everything is perfect");
        return Ok(())
    } else {
        msg!("Valid proof for everything is perfect");
        return Err(ProgramError::InvalidInstructionData);
    }
}
