use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_pubkey::Pubkey;
use solana_instruction::{ AccountMeta, Instruction };
use solana_message::Message;
use solana_transaction::Transaction;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use std::{ fs, str::FromStr };

fn load_svm() -> (LiteSVM, Pubkey) {
    let mut svm = LiteSVM::new();
    let program_id = Pubkey::from_str("4c3147pvchkSezQHN6z5oVSuGhidCNRHovScMih7NXHd").unwrap();
    let program_bytes = fs
        ::read("target/deploy/pinocchio_vault.so")
        .expect("run cargo build-sbf first");
    svm.add_program(&program_id, &program_bytes).unwrap();
    (svm, program_id)
}

fn deposit_ix(
    program_id: Pubkey,
    user: &Keypair,
    vault_pda: Pubkey,
    bump: u8,
    lamports: u64
) -> Instruction {
    let mut data = Vec::with_capacity(10);
    data.push(0u8); // discriminator
    data.push(bump);
    data.extend_from_slice(&lamports.to_le_bytes());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false)
        ],
        data,
    }
}

fn withdraw_ix(program_id: Pubkey, owner: &Keypair, vault_pda: Pubkey, bump: u8) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false)
        ],
        data: vec![1u8, bump],
    }
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

    let ix = deposit_ix(program_id, &user, vault_pda, bump, 1_00_000_000);
    let tx = Transaction::new(
        &[&user],
        Message::new(&[ix], Some(&user.pubkey())),
        svm.latest_blockhash()
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Deposit failed: {:?}", result.err());
    let vault_account = svm.get_account(&vault_pda).expect("Vault account not created");

    assert!(vault_account.lamports > 0);
    assert_eq!(vault_account.owner, SYSTEM_PROGRAM_ID); // owned by my program
}

#[test]
fn test_withdraw() {
    let (mut svm, program_id) = load_svm();

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", user.pubkey().as_ref()],
        &program_id
    );

    // first deposit
    let dep_ix = deposit_ix(program_id, &user, vault_pda, bump, 100_000_000);
    let dep_tx = Transaction::new(
        &[&user],
        Message::new(&[dep_ix], Some(&user.pubkey())),
        svm.latest_blockhash()
    );
    svm.send_transaction(dep_tx).expect("deposit failed");

    let balance_before = svm.get_account(&user.pubkey()).unwrap().lamports;

    // now withdraw
    let with_ix = withdraw_ix(program_id, &user, vault_pda, bump);
    let with_tx = Transaction::new(
        &[&user],
        Message::new(&[with_ix], Some(&user.pubkey())),
        svm.latest_blockhash()
    );
    let result = svm.send_transaction(with_tx);
    assert!(result.is_ok(), "Withdraw failed: {:#?}", result.err());

    let balance_after = svm.get_account(&user.pubkey()).unwrap().lamports;
    let vault_after = svm.get_account(&vault_pda);
    assert!(vault_after.is_none() || vault_after.unwrap().lamports == 0);
    // vault should be empty, user should have more lamports than before
    // assert_eq!(vault_after.lamports, 0);
    assert!(balance_after > balance_before);

    println!("withdraw passed");
    println!("balance before: {}", balance_before);
    println!("balance after:  {}", balance_after);
}

#[test]
fn test_attacker_cannot_withdraw() {
    let (mut svm, program_id) = load_svm();

    // legit user deposits
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", user.pubkey().as_ref()],
        &program_id
    );

    let dep_ix = deposit_ix(program_id, &user, vault_pda, bump, 100_000_000);
    let dep_tx = Transaction::new(
        &[&user],
        Message::new(&[dep_ix], Some(&user.pubkey())),
        svm.latest_blockhash()
    );
    svm.send_transaction(dep_tx).expect("deposit failed");

    // attacker tries to withdraw from user's vault
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();

    // attacker passes their own pubkey as owner but user's vault PDA
    // this should fail because PDA was derived from user's key not attacker's
    let attack_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(attacker.pubkey(), true), // attacker as owner
            AccountMeta::new(vault_pda, false), // but user's vault
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false)
        ],
        data: vec![1u8], // Withdraw discriminator
    };

    let attack_tx = Transaction::new(
        &[&attacker],
        Message::new(&[attack_ix], Some(&attacker.pubkey())),
        svm.latest_blockhash()
    );

    let result = svm.send_transaction(attack_tx);
    assert!(result.is_err(), "attacker should NOT be able to withdraw!");

    // vault should still have funds
    let vault = svm.get_account(&vault_pda).unwrap();
    assert!(vault.lamports > 0);

    println!("attacker rejected - vault still has {} lamports", vault.lamports);
}
