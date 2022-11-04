use cosmwasm_schema::{ cw_serde, QueryResponses };

#[cw_serde]
pub struct InstantiateMsg {
  pub bootstrap_data: String,
}

#[cw_serde]
pub enum ExecuteMsg {
  Update {
    update_data: String,
  },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  #[returns(StoreResponse)] Store {},
}

#[cw_serde]
pub struct StoreResponse {
  pub res: Vec<u8>,
}

impl From<Vec<u8>> for StoreResponse {
  fn from(store: Vec<u8>) -> StoreResponse {
    StoreResponse {
      res: store,
    }
  }
}
