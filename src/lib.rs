#![no_std]
#![allow(unused, dead_code)]
use pinocchio::{
    AccountView,
    Address,
    ProgramResult,
    entrypoint,
    error::ProgramError,
    nostd_panic_handler,
};

entrypoint!(process_instruction);
nostd_panic_handler!();

pub mod instructions;
pub mod state;
use instructions::{ Deposit, Withdraw };

pinocchio_pubkey::declare_id!("4c3147pvchkSezQHN6z5oVSuGhidCNRHovScMih7NXHd");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8]
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::new(data, accounts)?.process(),
        Some((Withdraw::DISCRIMINATOR, data)) => Withdraw::try_from((data, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
