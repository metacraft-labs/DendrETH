use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub vkey: Vec<u8>,
    pub current_header_hash: Vec<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Update {
        proof: Vec<u8>,
        new_optimistic_header_root: Vec<u8>,
        new_finalized_header_root: Vec<u8>,
        new_execution_state_root: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HeaderHash)]
    LastHeaderHash {},

    #[returns(HeaderHash)]
    LastFinalizedHeaderHash {},

    #[returns(HeaderHash)]
    LastExecStateRoot {},

    #[returns(HeaderHash)]
    HeaderHashBeforeNum {num:i32},

    #[returns(HeaderHash)]
    AllHeaderHashes {},

    #[returns(HeaderHash)]
    AllHeaderHashesOrdered {},

    #[returns(HeaderHash)]
    AllFinalizedHeaderHashes {},

    #[returns(HeaderHash)]
    AllFinalizedHeaderHashesOrdered {},

    #[returns(HeaderHash)]
    AllExecStateRoots {},

    #[returns(HeaderHash)]
    AllExecStateRootsOrdered {},
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
