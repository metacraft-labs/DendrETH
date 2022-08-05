// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

contract MockBLS {
    constructor() {}

    function fast_aggregate_verify(
        bytes[] calldata,
        bytes calldata,
        bytes calldata
    ) external pure returns (bool) {
        return true;
    }
}
