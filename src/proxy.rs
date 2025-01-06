use serde_json;
use crate::transactions;

pub fn build_send_transaction_payload(tx: &serde_json::Value) -> serde_json::Value {

    let payload = serde_json::json!({
        "tx": tx.to_string(),
    });

    return payload;

}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyInjectedTxResp {
    pub result: Option<transactions::InjectedTxResp>,
    pub error: Option<serde_json::Value>,
}
