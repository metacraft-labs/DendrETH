// SPDX-License-Identifier: MIT
pragma solidity 0.8.18;

interface IValidatorsAccumulator {
  event Deposited(
    bytes pubkey,
    bytes withdrawal_credentials,
    bytes signature,
    bytes32 deposit_data_root
  );

  function get_validators_accumulator() external view returns (bytes32 node);

  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawal_credentials,
    bytes calldata signature,
    bytes32 deposit_data_root
  ) external payable;
}
