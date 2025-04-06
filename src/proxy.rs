use crate::transactions;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub external_ip: String,
    pub external_port: u16,
    pub internal_ip: String,
    pub internal_port: u16,
    pub public_key: String,
    pub curve_public_key: String,
    pub status: String,
    pub app_data: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct NodeInfoResp {
    pub nodeInfo: NodeInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GetAccountResp {
    pub account: Option<serde_json::Value>,
}

pub fn build_send_transaction_payload(tx: &serde_json::Value) -> serde_json::Value {
    let payload = serde_json::json!({
        "tx": tx.to_string(),
    });

    payload
}

pub async fn get_random_node(url: &str) -> Result<NodeInfoResp, Box<dyn std::error::Error>> {
    let full_url = format!("{}/nodeinfo", url);

    match request(None, &full_url).await {
        Ok(res) => {
            let node_info: NodeInfoResp = serde_json::from_value(res)?;
            Ok(node_info)
        }
        Err(e) => Err(e),
    }
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
            Ok(json)
        }
        Err(e) => Err(Box::new(e)),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyInjectedTxResp {
    pub result: Option<transactions::InjectedTxResp>,
    pub error: Option<serde_json::Value>,
}
