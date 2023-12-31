use rotosports::pair::InstantiateMsg;
use rotosports::pair_bonded::{ExecuteMsg, QueryMsg};

use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg
    }
}
