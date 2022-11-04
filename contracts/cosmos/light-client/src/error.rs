use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
  #[error("{0}")] Std(#[from] StdError),

  #[error("Unauthorized")] Unauthorized {},
  // Add any other custom errors you like here.
  // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

#[no_mangle]
pub fn panic_on_error(text: *mut u8, len: usize) {
  panic!("{}", unsafe { String::from_raw_parts(text, len, 0) })
}
