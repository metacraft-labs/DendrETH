use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub vkey: Vec<u8>,
    pub currentHeader: Vec<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Update {
        proof: Vec<u8>,
        newHeader: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  #[returns(StoreResponse)] Header {},
}
#[cw_serde]
pub struct StoreResponse {
  pub res: Vec<u8>,
}

impl From<Vec<u8>> for StoreResponse {
  fn from(currentHeader: Vec<u8>) -> StoreResponse {
    StoreResponse {
      res: currentHeader,
    }
  }
}
