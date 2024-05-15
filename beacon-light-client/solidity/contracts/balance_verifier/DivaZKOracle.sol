interface IZKOracle is IZKOracleEvents, IZKOracleStructs {
  function getReport(
    uint256 slot
  )
    external
    view
    returns (
      bool success,
      uint256 clBalanceGwei,
      uint256 numValidators,
      uint256 exitedValidators,
      uint256 slashedValidators
    );
}

interface IZKOracleEvents {
  event Report(
    uint256 indexed slot,
    uint256 clBalanceGwei,
    uint256 numValidators,
    uint256 exitedValidators,
    uint256 slashedValidators
  );
}

interface IZKOracleStructs {
  struct Report {
    bool present;
    uint64 cBalanceGwei;
    uint64 numValidators;
    uint64 exitedValidators;
    uint64 slashedValidators;
  }
}
