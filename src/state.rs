pub struct VaultState {
    pub bump: u8,
}

impl VaultState {
    pub const LEN: usize = core::mem::size_of::<VaultState>();
}
