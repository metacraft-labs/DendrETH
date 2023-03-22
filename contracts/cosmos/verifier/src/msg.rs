use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub vkey: Vec<u8>,
    pub currentHeaderHash: Vec<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Update {
        proof: Vec<u8>,
        newOptimisticHeader: Vec<u8>,
        newFinalizedHeader: Vec<u8>,
        newExecutionStateRoot: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StoreResponse)]
    Header {},
}
#[cw_serde]
pub struct StoreResponse {
    pub res: Vec<u8>,
}

impl From<Vec<u8>> for StoreResponse {
    fn from(currentHeaderHash: Vec<u8>) -> StoreResponse {
        StoreResponse { res: currentHeaderHash }
    }
}
