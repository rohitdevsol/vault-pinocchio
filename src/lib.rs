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

// pub const ID: Pubkey = [
//     0x0f, 0x1e, 0x6b, 0x14, 0x21, 0xc0, 0x4a, 0x07, 0x04, 0x31, 0x26, 0x5c, 0x19, 0xc5, 0xbb, 0xee,
//     0x19, 0x92, 0xba, 0xe8, 0xaf, 0xd1, 0xcd, 0x07, 0x8e, 0xf8, 0xaf, 0x70, 0x47, 0xdc, 0x11, 0xf7,
// ];

pinocchio_pubkey::declare_id!("4c3147pvchkSezQHN6z5oVSuGhidCNRHovScMih7NXHd");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8]
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::new(data, accounts)?.process(),
        Some((Withdraw::DISCRIMINATOR, _)) => Withdraw::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
