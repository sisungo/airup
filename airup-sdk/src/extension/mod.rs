use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub class: u8,
    pub data: ciborium::Value,
}
impl Request {
    pub const CLASS_AIRUP_RPC: u8 = 1;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub data: ciborium::Value,
}
