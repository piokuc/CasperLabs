#![no_std]

extern crate contract_ffi;

use contract_ffi::contract_api;
use contract_ffi::contract_api::Error;
use contract_ffi::uref::URef;

#[repr(u32)]
enum Args {
    DoNothingURef = 0,
}

enum CustomError {
    MissingDoNothingURefArg = 0,
    InvalidDoNothingURefArg = 1,
    InvalidTURef = 2,
}

const ENTRY_FUNCTION_NAME: &str = "delegate";

#[no_mangle]
pub extern "C" fn delegate() {
    create_purse_01::delegate()
}

#[no_mangle]
pub extern "C" fn call() {
    let do_nothing_uref: URef = match contract_api::get_arg(Args::DoNothingURef as u32) {
        Some(Ok(data)) => data,
        Some(Err(_)) => {
            contract_api::revert(Error::User(CustomError::InvalidDoNothingURefArg as u16).into())
        }
        None => {
            contract_api::revert(Error::User(CustomError::MissingDoNothingURefArg as u16).into())
        }
    };

    let turef = contract_api::pointers::TURef::from_uref(do_nothing_uref).unwrap_or_else(|_| {
        contract_api::revert(Error::User(CustomError::InvalidTURef as u16).into())
    });

    // this should overwrite the previous contract obj with the new contract obj at the same uref
    contract_api::upgrade_contract_at_uref(ENTRY_FUNCTION_NAME, turef);
}
