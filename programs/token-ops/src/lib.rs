//! Token CPI benchmark program
//!
//! This program benchmarks the compute unit cost of various SPL Token
//! operations when invoked via CPI using pinocchio-token.
//!
//! Instruction format:
//! - Byte 0: Operation discriminator
//! - Remaining bytes: Operation-specific data
//!
//! Operations:
//! 0 = Transfer (amount: u64)
//!     Accounts: [source, destination, authority, token_program]
//!
//! 1 = MintTo (amount: u64)
//!     Accounts: [mint, destination, mint_authority, token_program]
//!
//! 2 = Burn (amount: u64)
//!     Accounts: [source, mint, authority, token_program]
//!
//! 3 = Approve (amount: u64)
//!     Accounts: [source, delegate, authority, token_program]
//!
//! 4 = Revoke
//!     Accounts: [source, authority, token_program]
//!
//! 5 = CloseAccount
//!     Accounts: [account, destination, authority, token_program]
//!
//! 6 = FreezeAccount
//!     Accounts: [account, mint, freeze_authority, token_program]
//!
//! 7 = ThawAccount
//!     Accounts: [account, mint, freeze_authority, token_program]
//!
//! 8 = TransferChecked (amount: u64, decimals: u8)
//!     Accounts: [source, mint, destination, authority, token_program]
//!
//! 9 = InitializeMint (decimals: u8, has_freeze_authority: u8)
//!     Accounts: [mint, rent_sysvar, mint_authority, freeze_authority?, token_program]
//!
//! 10 = InitializeMint2 (decimals: u8, has_freeze_authority: u8)
//!     Accounts: [mint, mint_authority, freeze_authority?, token_program]
//!
//! 11 = InitializeAccount
//!     Accounts: [account, mint, owner, rent_sysvar, token_program]
//!
//! 12 = InitializeAccount2
//!     Accounts: [account, mint, rent_sysvar, owner_address, token_program]
//!
//! 13 = InitializeAccount3
//!     Accounts: [account, mint, owner_address, token_program]
//!
//! 14 = SetAuthority (authority_type: u8, has_new_authority: u8)
//!     Accounts: [account, authority, new_authority?, token_program]

use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token::instructions::{
    Approve, Burn, CloseAccount, FreezeAccount, InitializeAccount, InitializeAccount2,
    InitializeAccount3, InitializeMint, InitializeMint2, MintTo, Revoke, SetAuthority,
    ThawAccount, Transfer, TransferChecked,
};

#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
    use pinocchio::{account::AccountView, entrypoint, Address, ProgramResult};

    entrypoint!(process_instruction);

    fn process_instruction(
        program_id: &Address,
        accounts: &[AccountView],
        instruction_data: &[u8],
    ) -> ProgramResult {
        super::process_instruction(program_id, accounts, instruction_data)
    }
}

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let operation = instruction_data[0];

    match operation {
        // Transfer
        0 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            Transfer {
                from: &accounts[0],
                to: &accounts[1],
                authority: &accounts[2],
                amount,
            }
            .invoke()
        }
        // MintTo
        1 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            MintTo {
                mint: &accounts[0],
                account: &accounts[1],
                mint_authority: &accounts[2],
                amount,
            }
            .invoke()
        }
        // Burn
        2 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            Burn {
                account: &accounts[0],
                mint: &accounts[1],
                authority: &accounts[2],
                amount,
            }
            .invoke()
        }
        // Approve
        3 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            Approve {
                source: &accounts[0],
                delegate: &accounts[1],
                authority: &accounts[2],
                amount,
            }
            .invoke()
        }
        // Revoke
        4 => Revoke {
            source: &accounts[0],
            authority: &accounts[1],
        }
        .invoke(),
        // CloseAccount
        5 => CloseAccount {
            account: &accounts[0],
            destination: &accounts[1],
            authority: &accounts[2],
        }
        .invoke(),
        // FreezeAccount
        6 => FreezeAccount {
            account: &accounts[0],
            mint: &accounts[1],
            freeze_authority: &accounts[2],
        }
        .invoke(),
        // ThawAccount
        7 => ThawAccount {
            account: &accounts[0],
            mint: &accounts[1],
            freeze_authority: &accounts[2],
        }
        .invoke(),
        // TransferChecked
        8 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let decimals = instruction_data[9];
            TransferChecked {
                from: &accounts[0],
                mint: &accounts[1],
                to: &accounts[2],
                authority: &accounts[3],
                amount,
                decimals,
            }
            .invoke()
        }
        // InitializeMint
        9 => {
            let decimals = instruction_data[1];
            let has_freeze_authority = instruction_data[2] != 0;
            let freeze_authority = if has_freeze_authority {
                Some(accounts[3].address())
            } else {
                None
            };
            InitializeMint {
                mint: &accounts[0],
                rent_sysvar: &accounts[1],
                decimals,
                mint_authority: accounts[2].address(),
                freeze_authority,
            }
            .invoke()
        }
        // InitializeMint2
        10 => {
            let decimals = instruction_data[1];
            let has_freeze_authority = instruction_data[2] != 0;
            let freeze_authority = if has_freeze_authority {
                Some(accounts[2].address())
            } else {
                None
            };
            InitializeMint2 {
                mint: &accounts[0],
                decimals,
                mint_authority: accounts[1].address(),
                freeze_authority,
            }
            .invoke()
        }
        // InitializeAccount
        11 => InitializeAccount {
            account: &accounts[0],
            mint: &accounts[1],
            owner: &accounts[2],
            rent_sysvar: &accounts[3],
        }
        .invoke(),
        // InitializeAccount2
        12 => InitializeAccount2 {
            account: &accounts[0],
            mint: &accounts[1],
            rent_sysvar: &accounts[2],
            owner: accounts[3].address(),
        }
        .invoke(),
        // InitializeAccount3
        13 => InitializeAccount3 {
            account: &accounts[0],
            mint: &accounts[1],
            owner: accounts[2].address(),
        }
        .invoke(),
        // SetAuthority
        14 => {
            let authority_type = instruction_data[1];
            let has_new_authority = instruction_data[2] != 0;
            let new_authority = if has_new_authority {
                Some(accounts[2].address())
            } else {
                None
            };
            SetAuthority {
                account: &accounts[0],
                authority: &accounts[1],
                authority_type: unsafe { core::mem::transmute(authority_type) },
                new_authority,
            }
            .invoke()
        }
        _ => Ok(()),
    }
}

pub const ID: [u8; 32] = [0x05; 32];
