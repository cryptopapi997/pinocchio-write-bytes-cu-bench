use litesvm::LiteSVM;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

const WRITE_LOOP_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0x03; 32]);
const WRITE_COPY_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0x04; 32]);
const TOKEN_OPS_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0x05; 32]);
const TOKEN_OPS_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0x06; 32]);

// Token-2022 program ID (TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb)
const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    6, 221, 246, 225, 238, 117, 143, 222, 24, 66, 93, 188, 228, 108, 205, 218,
    182, 26, 252, 77, 131, 185, 13, 39, 254, 189, 249, 40, 216, 161, 139, 252,
]);

fn main() {
    println!("\n=== write_bytes Benchmark (data serialization only) ===\n");
    println!(
        "{:>12} {:>12} {:>10} {:>10}",
        "Loop CU", "Copy CU", "Saved CU", "Saved %"
    );
    println!("{}", "-".repeat(48));
    benchmark_write_bytes();

    println!("\n=== Token CPI Benchmarks ===\n");
    benchmark_token_ops();

    println!("\n=== Token-2022 CPI Benchmarks ===\n");
    benchmark_token_2022_ops();
}

fn benchmark_write_bytes() {
    let loop_cu = run_write_benchmark(WRITE_LOOP_PROGRAM_ID, "write-loop");
    let copy_cu = run_write_benchmark(WRITE_COPY_PROGRAM_ID, "write-copy");

    let saved = loop_cu.saturating_sub(copy_cu);
    let percent = if loop_cu > 0 {
        (saved as f64 / loop_cu as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "{:>12} {:>12} {:>10} {:>9.1}%",
        loop_cu, copy_cu, saved, percent
    );
}

fn run_write_benchmark(program_id: Pubkey, program_name: &str) -> u64 {
    let program_path = format!("target/deploy/{}.so", program_name.replace('-', "_"));

    let program_bytes = match std::fs::read(&program_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to load {}: {}", program_path, e);
            eprintln!("Make sure to build with: cargo build-sbf");
            return 0;
        }
    };

    let mut svm = LiteSVM::new();
    svm.add_program(program_id, &program_bytes);

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Create one account for the benchmark
    let account_pubkey = Pubkey::new_unique();
    let account = Account {
        lamports: 1_000_000,
        data: vec![0u8; 100],
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(account_pubkey, account).unwrap();

    let instruction = Instruction {
        program_id,
        accounts: vec![AccountMeta {
            pubkey: account_pubkey,
            is_signer: false,
            is_writable: true,
        }],
        data: vec![],
    };

    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash,
    );

    match svm.send_transaction(tx) {
        Ok(tx_result) => tx_result.compute_units_consumed,
        Err(e) => {
            eprintln!("Transaction failed for {}: {:?}", program_name, e);
            e.meta.compute_units_consumed
        }
    }
}

fn benchmark_token_ops() {
    let token_ops_path = "target/deploy/token_ops.so";
    let token_ops_bytes = match std::fs::read(token_ops_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to load {}: {}", token_ops_path, e);
            eprintln!("Make sure to build with: cargo build-sbf --manifest-path programs/token-ops/Cargo.toml");
            return;
        }
    };

    println!(
        "{:<25} {:>12}",
        "Operation", "CU Consumed"
    );
    println!("{}", "-".repeat(38));

    // Benchmark Transfer
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::Transfer);
    println!("{:<25} {:>12}", "Transfer", cu);

    // Benchmark TransferChecked
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::TransferChecked);
    println!("{:<25} {:>12}", "TransferChecked", cu);

    // Benchmark MintTo
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::MintTo);
    println!("{:<25} {:>12}", "MintTo", cu);

    // Benchmark Burn
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::Burn);
    println!("{:<25} {:>12}", "Burn", cu);

    // Benchmark Approve
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::Approve);
    println!("{:<25} {:>12}", "Approve", cu);

    // Benchmark Revoke
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::Revoke);
    println!("{:<25} {:>12}", "Revoke", cu);

    // Benchmark FreezeAccount
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::FreezeAccount);
    println!("{:<25} {:>12}", "FreezeAccount", cu);

    // Benchmark ThawAccount
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::ThawAccount);
    println!("{:<25} {:>12}", "ThawAccount", cu);

    // Benchmark CloseAccount
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::CloseAccount);
    println!("{:<25} {:>12}", "CloseAccount", cu);

    // Benchmark InitializeMint
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::InitializeMint);
    println!("{:<25} {:>12}", "InitializeMint", cu);

    // Benchmark InitializeMint2
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::InitializeMint2);
    println!("{:<25} {:>12}", "InitializeMint2", cu);

    // Benchmark InitializeAccount
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::InitializeAccount);
    println!("{:<25} {:>12}", "InitializeAccount", cu);

    // Benchmark InitializeAccount2
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::InitializeAccount2);
    println!("{:<25} {:>12}", "InitializeAccount2", cu);

    // Benchmark InitializeAccount3
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::InitializeAccount3);
    println!("{:<25} {:>12}", "InitializeAccount3", cu);

    // Benchmark SetAuthority
    let cu = run_token_benchmark(&token_ops_bytes, TokenOp::SetAuthority);
    println!("{:<25} {:>12}", "SetAuthority", cu);
}

#[derive(Clone, Copy)]
enum TokenOp {
    Transfer,
    MintTo,
    Burn,
    Approve,
    Revoke,
    CloseAccount,
    FreezeAccount,
    ThawAccount,
    TransferChecked,
    InitializeMint,
    InitializeMint2,
    InitializeAccount,
    InitializeAccount2,
    InitializeAccount3,
    SetAuthority,
}

fn run_token_benchmark(token_ops_bytes: &[u8], op: TokenOp) -> u64 {
    let mut svm = LiteSVM::new();

    // Add SPL Token program
    svm.add_program(spl_token::ID, include_bytes!("spl_token.so"));

    // Add our token-ops benchmark program
    svm.add_program(TOKEN_OPS_PROGRAM_ID, token_ops_bytes);

    let payer = Keypair::new();
    let authority = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    // Create mint account (supply matches source token account balance)
    let mint = Pubkey::new_unique();
    let mint_data = create_mint_data(&authority.pubkey(), Some(&authority.pubkey()), 9, 1_000_000_000);
    svm.set_account(
        mint,
        Account {
            lamports: 1_000_000_000,
            data: mint_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create source token account
    let source_token = Pubkey::new_unique();
    let source_data = create_token_account_data(&mint, &authority.pubkey(), 1_000_000_000);
    svm.set_account(
        source_token,
        Account {
            lamports: 1_000_000_000,
            data: source_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create destination token account
    let dest_token = Pubkey::new_unique();
    let dest_data = create_token_account_data(&mint, &authority.pubkey(), 0);
    svm.set_account(
        dest_token,
        Account {
            lamports: 1_000_000_000,
            data: dest_data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create delegate account (for approve/revoke)
    let delegate = Pubkey::new_unique();

    // Build instruction based on operation
    let (accounts, data, needs_authority_signer) = match op {
        TokenOp::Transfer => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let mut data = vec![0u8]; // discriminator for Transfer
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::TransferChecked => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let mut data = vec![8u8]; // discriminator for TransferChecked
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.push(9); // decimals
            (accounts, data, true)
        }
        TokenOp::MintTo => {
            let accounts = vec![
                AccountMeta::new(mint, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let mut data = vec![1u8]; // discriminator for MintTo
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Burn => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let mut data = vec![2u8]; // discriminator for Burn
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Approve => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(delegate, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let mut data = vec![3u8]; // discriminator for Approve
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Revoke => {
            // First approve a delegate, then revoke
            {
                let approve_accounts = vec![
                    AccountMeta::new(source_token, false),
                    AccountMeta::new_readonly(delegate, false),
                    AccountMeta::new_readonly(authority.pubkey(), true),
                    AccountMeta::new_readonly(spl_token::ID, false),
                ];
                let mut approve_data = vec![3u8];
                approve_data.extend_from_slice(&1000u64.to_le_bytes());

                let instruction = Instruction {
                    program_id: TOKEN_OPS_PROGRAM_ID,
                    accounts: approve_accounts,
                    data: approve_data,
                };
                let blockhash = svm.latest_blockhash();
                let tx = Transaction::new_signed_with_payer(
                    &[instruction],
                    Some(&payer.pubkey()),
                    &[&payer, &authority],
                    blockhash,
                );
                let _ = svm.send_transaction(tx);
            }

            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![4u8]; // discriminator for Revoke
            (accounts, data, true)
        }
        TokenOp::FreezeAccount => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![6u8]; // discriminator for FreezeAccount
            (accounts, data, true)
        }
        TokenOp::ThawAccount => {
            // First freeze the account
            {
                let freeze_accounts = vec![
                    AccountMeta::new(source_token, false),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new_readonly(authority.pubkey(), true),
                    AccountMeta::new_readonly(spl_token::ID, false),
                ];
                let freeze_data = vec![6u8];

                let instruction = Instruction {
                    program_id: TOKEN_OPS_PROGRAM_ID,
                    accounts: freeze_accounts,
                    data: freeze_data,
                };
                let blockhash = svm.latest_blockhash();
                let tx = Transaction::new_signed_with_payer(
                    &[instruction],
                    Some(&payer.pubkey()),
                    &[&payer, &authority],
                    blockhash,
                );
                let _ = svm.send_transaction(tx);
            }

            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![7u8]; // discriminator for ThawAccount
            (accounts, data, true)
        }
        TokenOp::CloseAccount => {
            // Create a fresh token account with zero balance for closing
            let close_token = Pubkey::new_unique();
            let close_data = create_token_account_data(&mint, &authority.pubkey(), 0);
            svm.set_account(
                close_token,
                Account {
                    lamports: 1_000_000_000,
                    data: close_data,
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(close_token, false),
                AccountMeta::new(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![5u8]; // discriminator for CloseAccount
            (accounts, data, true)
        }
        TokenOp::InitializeMint => {
            // Create uninitialized mint account
            let new_mint = Pubkey::new_unique();
            svm.set_account(
                new_mint,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 82], // Mint size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_mint, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false), // freeze authority
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![9u8, 9, 1]; // discriminator, decimals, has_freeze_authority
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeMint2 => {
            // Create uninitialized mint account
            let new_mint = Pubkey::new_unique();
            svm.set_account(
                new_mint,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 82], // Mint size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false), // freeze authority
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![10u8, 9, 1]; // discriminator, decimals, has_freeze_authority
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![11u8]; // discriminator for InitializeAccount
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount2 => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(authority.pubkey(), false), // owner address
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![12u8]; // discriminator for InitializeAccount2
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount3 => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: spl_token::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false), // owner address
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![13u8]; // discriminator for InitializeAccount3
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::SetAuthority => {
            let new_authority = Pubkey::new_unique();
            let accounts = vec![
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(new_authority, false), // new authority
                AccountMeta::new_readonly(spl_token::ID, false),
            ];
            let data = vec![14u8, 0, 1]; // discriminator, authority_type (MintTokens=0), has_new_authority
            (accounts, data, true) // authority signer needed
        }
    };

    let instruction = Instruction {
        program_id: TOKEN_OPS_PROGRAM_ID,
        accounts,
        data,
    };

    let blockhash = svm.latest_blockhash();
    let tx = if needs_authority_signer {
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            blockhash,
        )
    } else {
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        )
    };

    match svm.send_transaction(tx) {
        Ok(tx_result) => tx_result.compute_units_consumed,
        Err(e) => {
            eprintln!("Transaction failed for {:?}: {:?}", op as u8, e);
            e.meta.compute_units_consumed
        }
    }
}

fn benchmark_token_2022_ops() {
    let token_ops_2022_path = "target/deploy/token_ops_2022.so";
    let token_ops_2022_bytes = match std::fs::read(token_ops_2022_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to load {}: {}", token_ops_2022_path, e);
            eprintln!("Make sure to build with: cargo build-sbf --manifest-path programs/token-ops-2022/Cargo.toml");
            return;
        }
    };

    println!(
        "{:<25} {:>12}",
        "Operation", "CU Consumed"
    );
    println!("{}", "-".repeat(38));

    // Benchmark Transfer
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::Transfer);
    println!("{:<25} {:>12}", "Transfer", cu);

    // Benchmark TransferChecked
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::TransferChecked);
    println!("{:<25} {:>12}", "TransferChecked", cu);

    // Benchmark MintTo
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::MintTo);
    println!("{:<25} {:>12}", "MintTo", cu);

    // Benchmark Burn
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::Burn);
    println!("{:<25} {:>12}", "Burn", cu);

    // Benchmark Approve
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::Approve);
    println!("{:<25} {:>12}", "Approve", cu);

    // Benchmark Revoke
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::Revoke);
    println!("{:<25} {:>12}", "Revoke", cu);

    // Benchmark FreezeAccount
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::FreezeAccount);
    println!("{:<25} {:>12}", "FreezeAccount", cu);

    // Benchmark ThawAccount
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::ThawAccount);
    println!("{:<25} {:>12}", "ThawAccount", cu);

    // Benchmark CloseAccount
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::CloseAccount);
    println!("{:<25} {:>12}", "CloseAccount", cu);

    // Benchmark InitializeMint
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::InitializeMint);
    println!("{:<25} {:>12}", "InitializeMint", cu);

    // Benchmark InitializeMint2
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::InitializeMint2);
    println!("{:<25} {:>12}", "InitializeMint2", cu);

    // Benchmark InitializeAccount
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::InitializeAccount);
    println!("{:<25} {:>12}", "InitializeAccount", cu);

    // Benchmark InitializeAccount2
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::InitializeAccount2);
    println!("{:<25} {:>12}", "InitializeAccount2", cu);

    // Benchmark InitializeAccount3
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::InitializeAccount3);
    println!("{:<25} {:>12}", "InitializeAccount3", cu);

    // Benchmark SetAuthority
    let cu = run_token_2022_benchmark(&token_ops_2022_bytes, TokenOp::SetAuthority);
    println!("{:<25} {:>12}", "SetAuthority", cu);
}

fn run_token_2022_benchmark(token_ops_bytes: &[u8], op: TokenOp) -> u64 {
    let mut svm = LiteSVM::new();

    // Add SPL Token-2022 program
    svm.add_program(TOKEN_2022_PROGRAM_ID, include_bytes!("spl_token_2022.so"));

    // Add our token-ops-2022 benchmark program
    svm.add_program(TOKEN_OPS_2022_PROGRAM_ID, token_ops_bytes);

    let payer = Keypair::new();
    let authority = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    // Create mint account (supply matches source token account balance)
    let mint = Pubkey::new_unique();
    let mint_data = create_mint_data(&authority.pubkey(), Some(&authority.pubkey()), 9, 1_000_000_000);
    svm.set_account(
        mint,
        Account {
            lamports: 1_000_000_000,
            data: mint_data,
            owner: TOKEN_2022_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create source token account
    let source_token = Pubkey::new_unique();
    let source_data = create_token_account_data(&mint, &authority.pubkey(), 1_000_000_000);
    svm.set_account(
        source_token,
        Account {
            lamports: 1_000_000_000,
            data: source_data,
            owner: TOKEN_2022_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create destination token account
    let dest_token = Pubkey::new_unique();
    let dest_data = create_token_account_data(&mint, &authority.pubkey(), 0);
    svm.set_account(
        dest_token,
        Account {
            lamports: 1_000_000_000,
            data: dest_data,
            owner: TOKEN_2022_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    // Create delegate account (for approve/revoke)
    let delegate = Pubkey::new_unique();

    // Build instruction based on operation
    let (accounts, data, needs_authority_signer) = match op {
        TokenOp::Transfer => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let mut data = vec![0u8]; // discriminator for Transfer
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::TransferChecked => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let mut data = vec![8u8]; // discriminator for TransferChecked
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.push(9); // decimals
            (accounts, data, true)
        }
        TokenOp::MintTo => {
            let accounts = vec![
                AccountMeta::new(mint, false),
                AccountMeta::new(dest_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let mut data = vec![1u8]; // discriminator for MintTo
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Burn => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let mut data = vec![2u8]; // discriminator for Burn
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Approve => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(delegate, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let mut data = vec![3u8]; // discriminator for Approve
            data.extend_from_slice(&1000u64.to_le_bytes());
            (accounts, data, true)
        }
        TokenOp::Revoke => {
            // First approve a delegate, then revoke
            {
                let approve_accounts = vec![
                    AccountMeta::new(source_token, false),
                    AccountMeta::new_readonly(delegate, false),
                    AccountMeta::new_readonly(authority.pubkey(), true),
                    AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
                ];
                let mut approve_data = vec![3u8];
                approve_data.extend_from_slice(&1000u64.to_le_bytes());

                let instruction = Instruction {
                    program_id: TOKEN_OPS_2022_PROGRAM_ID,
                    accounts: approve_accounts,
                    data: approve_data,
                };
                let blockhash = svm.latest_blockhash();
                let tx = Transaction::new_signed_with_payer(
                    &[instruction],
                    Some(&payer.pubkey()),
                    &[&payer, &authority],
                    blockhash,
                );
                let _ = svm.send_transaction(tx);
            }

            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![4u8]; // discriminator for Revoke
            (accounts, data, true)
        }
        TokenOp::FreezeAccount => {
            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![6u8]; // discriminator for FreezeAccount
            (accounts, data, true)
        }
        TokenOp::ThawAccount => {
            // First freeze the account
            {
                let freeze_accounts = vec![
                    AccountMeta::new(source_token, false),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new_readonly(authority.pubkey(), true),
                    AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
                ];
                let freeze_data = vec![6u8];

                let instruction = Instruction {
                    program_id: TOKEN_OPS_2022_PROGRAM_ID,
                    accounts: freeze_accounts,
                    data: freeze_data,
                };
                let blockhash = svm.latest_blockhash();
                let tx = Transaction::new_signed_with_payer(
                    &[instruction],
                    Some(&payer.pubkey()),
                    &[&payer, &authority],
                    blockhash,
                );
                let _ = svm.send_transaction(tx);
            }

            let accounts = vec![
                AccountMeta::new(source_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![7u8]; // discriminator for ThawAccount
            (accounts, data, true)
        }
        TokenOp::CloseAccount => {
            // Create a fresh token account with zero balance for closing
            let close_token = Pubkey::new_unique();
            let close_data = create_token_account_data(&mint, &authority.pubkey(), 0);
            svm.set_account(
                close_token,
                Account {
                    lamports: 1_000_000_000,
                    data: close_data,
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(close_token, false),
                AccountMeta::new(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![5u8]; // discriminator for CloseAccount
            (accounts, data, true)
        }
        TokenOp::InitializeMint => {
            // Create uninitialized mint account
            let new_mint = Pubkey::new_unique();
            svm.set_account(
                new_mint,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 82], // Mint size
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_mint, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false), // freeze authority
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![9u8, 9, 1]; // discriminator, decimals, has_freeze_authority
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeMint2 => {
            // Create uninitialized mint account
            let new_mint = Pubkey::new_unique();
            svm.set_account(
                new_mint,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 82], // Mint size
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(authority.pubkey(), false), // freeze authority
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![10u8, 9, 1]; // discriminator, decimals, has_freeze_authority
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![11u8]; // discriminator for InitializeAccount
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount2 => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(authority.pubkey(), false), // owner address
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![12u8]; // discriminator for InitializeAccount2
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::InitializeAccount3 => {
            // Create uninitialized token account
            let new_token = Pubkey::new_unique();
            svm.set_account(
                new_token,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![0u8; 165], // Token account size
                    owner: TOKEN_2022_PROGRAM_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .unwrap();

            let accounts = vec![
                AccountMeta::new(new_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), false), // owner address
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![13u8]; // discriminator for InitializeAccount3
            (accounts, data, false) // no authority signer needed
        }
        TokenOp::SetAuthority => {
            let new_authority = Pubkey::new_unique();
            let accounts = vec![
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(authority.pubkey(), true),
                AccountMeta::new_readonly(new_authority, false), // new authority
                AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
            ];
            let data = vec![14u8, 0, 1]; // discriminator, authority_type (MintTokens=0), has_new_authority
            (accounts, data, true) // authority signer needed
        }
    };

    let instruction = Instruction {
        program_id: TOKEN_OPS_2022_PROGRAM_ID,
        accounts,
        data,
    };

    let blockhash = svm.latest_blockhash();
    let tx = if needs_authority_signer {
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            blockhash,
        )
    } else {
        Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        )
    };

    match svm.send_transaction(tx) {
        Ok(tx_result) => tx_result.compute_units_consumed,
        Err(e) => {
            eprintln!("Transaction failed for {:?} (Token-2022): {:?}", op as u8, e);
            e.meta.compute_units_consumed
        }
    }
}

/// Creates mint account data in SPL Token format
fn create_mint_data(mint_authority: &Pubkey, freeze_authority: Option<&Pubkey>, decimals: u8, supply: u64) -> Vec<u8> {
    let mut data = vec![0u8; 82]; // SPL Token Mint size

    // mint_authority (COption<Pubkey>)
    data[0..4].copy_from_slice(&1u32.to_le_bytes()); // Some
    data[4..36].copy_from_slice(mint_authority.as_ref());

    // supply (u64)
    data[36..44].copy_from_slice(&supply.to_le_bytes());

    // decimals (u8)
    data[44] = decimals;

    // is_initialized (bool)
    data[45] = 1;

    // freeze_authority (COption<Pubkey>)
    if let Some(auth) = freeze_authority {
        data[46..50].copy_from_slice(&1u32.to_le_bytes()); // Some
        data[50..82].copy_from_slice(auth.as_ref());
    } else {
        data[46..50].copy_from_slice(&0u32.to_le_bytes()); // None
    }

    data
}

/// Creates token account data in SPL Token format
fn create_token_account_data(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Vec<u8> {
    let mut data = vec![0u8; 165]; // SPL Token Account size

    // mint (Pubkey)
    data[0..32].copy_from_slice(mint.as_ref());

    // owner (Pubkey)
    data[32..64].copy_from_slice(owner.as_ref());

    // amount (u64)
    data[64..72].copy_from_slice(&amount.to_le_bytes());

    // delegate (COption<Pubkey>) - None
    data[72..76].copy_from_slice(&0u32.to_le_bytes());

    // state (AccountState) - 1 = Initialized
    data[108] = 1;

    // is_native (COption<u64>) - None
    data[109..113].copy_from_slice(&0u32.to_le_bytes());

    // delegated_amount (u64)
    data[121..129].copy_from_slice(&0u64.to_le_bytes());

    // close_authority (COption<Pubkey>) - None
    data[129..133].copy_from_slice(&0u32.to_le_bytes());

    data
}
