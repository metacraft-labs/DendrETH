pub struct BalanceVerificationConfig {
    pub redis_connection: String,
    pub circuit_level: u64,
    pub stop_after: u64,
    pub lease_for: u64,
    pub run_for_minutes: Option<u64>,
    pub preserve_intermediary_proofs: bool,
}
