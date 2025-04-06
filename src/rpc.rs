use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Consensor {
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

    payload
}

pub fn build_get_account_payload(account_id: &str) -> serde_json::Value {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lib_getAccount",
        "params": [account_id],
        "id": 1,
    });

    payload
}

pub fn build_get_nodelist_payload() -> serde_json::Value {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lib_getNodeList",
        "params": [],
        "id": 1,
    });

    payload
}

pub async fn request(
    serde_json_payload: &serde_json::Value,
    url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.post(url).json(&serde_json_payload).send().await;

    match res {
        Ok(res) => {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            Ok(json)
        }
        Err(e) => Err(Box::new(e)),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: i32,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}
