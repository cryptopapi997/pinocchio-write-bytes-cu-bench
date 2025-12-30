use core::mem::MaybeUninit;
use pinocchio::{account::AccountView, Address, ProgramResult};

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

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[inline(always)]
fn write_bytes_copy(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    let len = destination.len().min(source.len());
    unsafe {
        core::ptr::copy_nonoverlapping(
            source.as_ptr(),
            destination.as_mut_ptr() as *mut u8,
            len,
        );
    }
}

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {

    let account = &accounts[0];

    // Transfer
    let mut data1 = [UNINIT_BYTE; 9];
    write_bytes_copy(&mut data1[0..1], &[3u8]); // discriminator
    write_bytes_copy(&mut data1[1..9], &12345678u64.to_le_bytes()); // amount

    // Initialize mint
    let mut data2 = [UNINIT_BYTE; 67];
    write_bytes_copy(&mut data2[0..1], &[0u8]); // discriminator
    write_bytes_copy(&mut data2[1..2], &[9u8]); // decimals
    write_bytes_copy(&mut data2[2..34], account.address().as_ref()); // mint authority
    write_bytes_copy(&mut data2[34..35], &[1u8]); // has freeze authority
    write_bytes_copy(&mut data2[35..67], account.address().as_ref()); // freeze authority

    core::hint::black_box(&data1);
    core::hint::black_box(&data2);

    Ok(())
}

pub const ID: [u8; 32] = [0x04; 32];
