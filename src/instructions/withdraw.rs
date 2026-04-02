use pinocchio::{ AccountView, ProgramResult, cpi::{ Seed, Signer }, error::ProgramError };
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::Transfer;

pub struct Withdraw<'a> {
    pub owner: &'a AccountView,
    pub vault: &'a AccountView,
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&self) -> ProgramResult {
        // read bump from vault
        let bump = unsafe { self.vault.borrow_unchecked()[0] };
        let bump_binding = [bump];

        let expected_vault = derive_address(
            &[b"vault", self.owner.address().as_ref()],
            Some(bump),
            &crate::ID
        );

        if self.vault.address().as_ref().ne(&expected_vault) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // transfer lamports
        let lamports = self.vault.lamports();

        //signer seeds
        let signer_seeds = [
            Seed::from(b"vault"),
            Seed::from(self.owner.address().as_ref()),
            Seed::from(&bump_binding[..]),
        ];

        let signers = [Signer::from(&signer_seeds[..])];

        (Transfer {
            from: self.vault,
            to: self.owner,
            lamports,
        }).invoke_signed(&signers)?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountView]> for Withdraw<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !vault.owned_by(&crate::ID.into()) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { owner, vault })
    }
}
