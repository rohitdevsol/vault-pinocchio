use pinocchio::{
    AccountView,
    Address,
    ProgramResult,
    cpi::{ Seed, Signer },
    error::ProgramError,
    sysvars::{ Sysvar, rent::Rent },
};
use pinocchio_pubkey::{ derive_address };
use pinocchio_system::instructions::{ CreateAccount, Transfer };
use crate::state::VaultState;

// deposit accounts

pub struct DepositAccounts<'a> {
    pub owner: &'a AccountView,
    pub vault: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DepositAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self {
            owner,
            vault,
        })
    }
}
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub bump: u8,
    pub lamports: u64,
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    // named constructor instead of TryFrom to avoid blanket impl conflicts
    pub fn new(data: &'a [u8], accounts: &'a [AccountView]) -> Result<Self, ProgramError> {
        // instruction data layout: [bump: u8 (1 byte), lamports: u64 (8 bytes)]
        if data.len() < 9 {
            return Err(ProgramError::InvalidInstructionData);
        }
        let accounts = DepositAccounts::try_from(accounts)?;
        let bump = data[0];
        let lamports = u64::from_le_bytes(data[1..9].try_into().unwrap());
        if lamports.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { accounts, bump, lamports })
    }

    pub fn process(&mut self) -> ProgramResult {
        // derive the expected vault using the bump
        // let bump = self.bump;
        // let bump_binding = [bump];

        let expected_vault = derive_address(
            &[b"vault", self.accounts.owner.address().as_ref()],
            Some(self.bump),
            &crate::ID
        );

        if self.accounts.vault.address().as_ref().ne(&expected_vault) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // // vault account(create) space =1 byte for bump storage
        // // let rent = Rent::get()?;
        // let lamports_for_rent = Rent::get()?.try_minimum_balance(VaultState::LEN)?;

        // let signer_seeds = [
        //     Seed::from(b"vault"),
        //     Seed::from(self.accounts.owner.address().as_ref()),
        //     Seed::from(&bump_binding[..]),
        // ];

        // let signers = [Signer::from(&signer_seeds[..])];
        // let program_id: pinocchio::Address = crate::ID.into();

        // (CreateAccount {
        //     from: self.accounts.owner,
        //     to: self.accounts.vault,
        //     space: VaultState::LEN as u64,
        //     lamports: lamports_for_rent,
        //     owner: &program_id,
        // }).invoke_signed(&signers)?;

        (Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.lamports,
        }).invoke()?;

        Ok(())
    }
}
