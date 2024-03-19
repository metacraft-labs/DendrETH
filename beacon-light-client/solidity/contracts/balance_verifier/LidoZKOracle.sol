pragma solidity ^0.8.19;

interface LidoZKOracle {
    function getReport(uint256 refSlot) external view returns  (
            bool success,
            uint256 clBalanceGwei,
            uint256 numValidators,
            uint256 exitedValidators
	);
}
