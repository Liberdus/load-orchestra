use crate::transactions;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GetAccountResp {
    pub account: Option<serde_json::Value>,
}

pub fn build_send_transaction_payload(tx: &serde_json::Value) -> serde_json::Value {
    let payload = serde_json::json!({
        "tx": tx.to_string(),
    });

    return payload;
}

pub async fn request(
    serde_json_payload: Option<&serde_json::Value>,
    url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = match serde_json_payload {
        Some(payload) => client.post(url).json(&payload).send().await,
        None => client.get(url).send().await,
    };

    match res {
        Ok(res) => {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            return Ok(json);
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyInjectedTxResp {
    pub result: Option<transactions::InjectedTxResp>,
    pub error: Option<serde_json::Value>,
}
