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

fn main() {
    println!(
        "{:>12} {:>12} {:>10} {:>10}",
        "Loop CU", "Copy CU", "Saved CU", "Saved %"
    );
    println!("{}", "-".repeat(64));

    benchmark_write_bytes();
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
