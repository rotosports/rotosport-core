use cosmwasm_schema::write_api;

use rotosports::pair::InstantiateMsg;
use rotosports::pair_bonded::{ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
