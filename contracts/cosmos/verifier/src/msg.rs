use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub vkey: i32,
    pub currentHeader: i32,
}

#[cw_serde]
pub enum ExecuteMsg {
    //UpdateAndValidation {
    //    new_header_hash1: Uint256,
      //  new_header_hash2: Uint256,
        //Proof??
    //},
    Update {
        //update_data: i32,
        proofInput: i32,
        newHeader: i32,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  #[returns(StoreResponse)] Store {},
  #[returns(StoreResponse)] Header {},
}
#[cw_serde]
pub struct StoreResponse {
  pub res: i32,
}

impl From<i32> for StoreResponse {
  fn from(store: i32) -> StoreResponse {
    StoreResponse {
      res: store,
    }
  }
}
// #[cw_serde]
// pub struct StoreResponse2 {
//   pub res: i32,
// }

// impl From<i32> for StoreResponse2 {
//   fn from(lcs: i32) -> StoreResponse2 {
//     StoreResponse2 {
//       res: lcs,
//     }
//   }
// }
