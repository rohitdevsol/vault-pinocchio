use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_pubkey::Pubkey;
use solana_instruction::{ AccountMeta, Instruction };
use solana_message::Message;
use solana_transaction::Transaction;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use std::fs;

fn load_svm() -> (LiteSVM, Pubkey) {
    let mut svm = LiteSVM::new();
    let program_id = Pubkey::new_unique();
    let program_bytes = fs
        ::read("target/deploy/pinocchio_vault.so")
        .expect("run cargo build-sbf first");
    svm.add_program(&program_id, &program_bytes).unwrap();
    (svm, program_id)
}

#[test]
fn test_deposit() {
    let (mut svm, program_id) = load_svm();
    // create a new user
    let user = Keypair::new();

    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    // derive the vault pda
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", user.pubkey().as_ref()],
        &program_id
    );

    // instruction data layout: [discriminator(1), bump(1), lamports(8)]

    let deposit_lamports: u64 = 100_000_000; // 0.1 SOL
    let mut data = Vec::with_capacity(10);
    data.push(0u8); // discriminator = Deposit
    data.push(bump);
    data.extend_from_slice(&deposit_lamports.to_le_bytes()); // lamports as little-endian u64

    // building the instruction
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true), // owner — signer, writable
            AccountMeta::new(vault_pda, false), // vault PDA — writable, not signer
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false) // system program
        ],
        data,
    };
    let tx = Transaction::new(
        &[&user],
        Message::new(&[ix], Some(&user.pubkey())),
        svm.latest_blockhash()
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Deposit failed: {:?}", result.err());
    let vault_account = svm.get_account(&vault_pda).expect("Vault account not created");

    assert!(vault_account.lamports > 0);
    assert_eq!(vault_account.data[0], bump); // bump stored at byte 0
    assert_eq!(vault_account.owner, program_id); // owned by my program
}
