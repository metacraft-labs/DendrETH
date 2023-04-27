use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub vkey_path: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Update {
    proof_path: String,
    input_path: String,
    vkey_path: String,
}}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HeaderHash)]
    LastHeaderHash {},
}

#[cw_serde]
pub struct HeaderHash {
    pub res: Vec<u8>,
}

impl From<Vec<u8>> for HeaderHash {
    fn from(header_hash: Vec<u8>) -> HeaderHash {
        HeaderHash { res: header_hash }
    }
}
