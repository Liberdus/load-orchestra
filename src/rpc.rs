use serde_json;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Consensor{
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub publicKey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub rng_bias: Option<f64>,
}

pub fn build_send_transaction_payload(tx: &serde_json::Value) -> serde_json::Value {

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lib_sendTransaction",
        "params": [tx.to_string()],
        "id": 1,
    });

    return payload;

}

pub fn get_account(account_id: &str) -> serde_json::Value {

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lib_getAccount",
        "params": [account_id],
        "id": 1,
    });

    return payload;

}

pub fn get_nodelist() -> serde_json::Value {

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lib_getNodeList",
        "params": [],
        "id": 1,
    });

    return payload;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: i32,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError{
    pub code: i32,
    pub message: String,
}



