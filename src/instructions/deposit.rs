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
pub struct Deposit<'a> {
    pub owner: &'a AccountView,
    pub vault: &'a AccountView,
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
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump = data[0];
        let lamports = u64::from_le_bytes(data[1..9].try_into().unwrap());

        Ok(Self { owner, vault, bump, lamports })
    }

    pub fn process(&mut self) -> ProgramResult {
        // derive the expected vault using the bump
        let bump = self.bump;
        let bump_binding = [bump];

        let expected_vault = derive_address(
            &[b"vault", self.owner.address().as_ref()],
            Some(bump),
            &crate::ID
        );

        if self.vault.address().as_ref().ne(&expected_vault) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // vault account(create) space =1 byte for bump storage
        let rent = Rent::get()?;
        let lamports_for_rent = rent.minimum_balance(VaultState::LEN);

        let signer_seeds = [
            Seed::from(b"vault"),
            Seed::from(self.owner.address().as_ref()),
            Seed::from(&bump_binding[..]),
        ];

        let signers = [Signer::from(&signer_seeds[..])];
        let program_id: pinocchio::Address = crate::ID.into();

        (CreateAccount {
            from: self.owner,
            to: self.vault,
            space: VaultState::LEN as u64,
            lamports: lamports_for_rent,
            owner: &program_id,
        }).invoke_signed(&signers)?;

        unsafe {
            let ptr = self.vault.borrow_unchecked().as_ptr() as *mut u8;
            *ptr = self.bump;
        }

        (Transfer {
            from: self.owner,
            to: self.vault,
            lamports: self.lamports,
        }).invoke()?;

        Ok(())
    }
}
