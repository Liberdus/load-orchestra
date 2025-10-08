use crate::{cli, crypto, proxy, utils};
use alloy::signers::{k256::ecdsa::SigningKey, local::LocalSigner, SignerSync};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static NETWORK_ID: OnceLock<String> = OnceLock::new();

fn get_network_id() -> &'static str {
    NETWORK_ID.get_or_init(|| {
        dotenvy::dotenv().ok();
        std::env::var("NETWORK_ID")
            .unwrap_or_else(|_| "liberdus-default".to_string())
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct InjectedTxResp {
    pub reason: String,
    pub status: u32,
    pub success: bool,
    pub txId: Option<String>,
}

pub enum LiberdusTransactions {
    Register(RegisterTransaction),
    Transfer(TransferTransaction),
    Message(MessageTransaction),
    DepositStake(DepositStakeTransaction),
    ChangeConfig(ChangeConfigTransaction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeConfigTransaction {
    pub from: String,
    pub cycle: i64,
    pub config: String,
    pub networkId: String,
    pub sign: ShardusSignature,

    #[serde(rename = "type")]
    pub transaction_type: String,

    pub timestamp: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositStakeTransaction {
    pub nominee: String,
    pub stake: ShardusBigIntSerialized,
    pub nominator: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub networkId: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShardusSignature {
    owner: String,
    sig: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct RegisterTransaction {
    pub aliasHash: String,
    pub from: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub alias: String,
    pub publicKey: String,
    pub networkId: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriendTransaction {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub alias: String,
    pub networkId: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct MessageTransaction {
    pub from: String,
    pub to: String,
    pub amount: ShardusBigIntSerialized,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub chatId: String,
    pub message: String,
    pub networkId: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferTransaction {
    pub from: String,
    pub to: String,
    pub amount: ShardusBigIntSerialized,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub networkId: String,
    pub timestamp: u128,
    pub memo: Option<String>,
    #[allow(non_snake_case)]
    pub chatId: String,
    pub sign: ShardusSignature,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShardusBigIntSerialized {
    pub dataType: String,
    pub value: String,
}

pub fn build_change_config_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    cycle: i64,
    config: &String,
) -> ChangeConfigTransaction {
    let from = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let tx = serde_json::json!({
        "from": utils::to_shardus_address(&from),
        "cycle": cycle,
        "type": "change_config".to_string(),
        "config": config,
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    ChangeConfigTransaction {
        from: utils::to_shardus_address(&from),
        cycle,
        config: config.clone(),
        networkId: get_network_id().to_string(),
        transaction_type: "change_config".to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn build_message_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    to: &alloy::primitives::Address,
    message: &String,
) -> MessageTransaction {
    let from = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let chat_id = {
        // lexically sort the two addresses, smaller address first
        let from_address = utils::to_shardus_address(&from);
        let to = utils::to_shardus_address(&to.to_string());

        let mut addresses = [from_address, to];
        addresses.sort();

        shardus_crypto
            .hash(&addresses.join("").into_bytes(), crypto::Format::Hex)
            .to_string()
    };
    let tx = serde_json::json!({
        "from": utils::to_shardus_address(&from),
        "amount": serde_json::json!({
            "dataType": "bi",
            "value": "25000000000000000000",
        }),
        "to": utils::to_shardus_address(&to.to_string()),
        "type": "message",
        "chatId": chat_id,
        "message": message,
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    MessageTransaction {
        amount: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: "25000000000000000000".to_string(),
        },
        from: utils::to_shardus_address(&from),
        to: utils::to_shardus_address(&to.to_string()),
        transaction_type: "message".to_string(),
        chatId: chat_id,
        message: message.clone(),
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
    }
}

#[allow(dead_code)]
pub fn build_friend_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    to: &alloy::primitives::Address,
    alias: &String,
) -> FriendTransaction {
    let from = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tx = serde_json::json!({
        "from": utils::to_shardus_address(&from),
        "to": utils::to_shardus_address(&to.to_string()),
        "type": "friend",
        "alias": alias,
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    FriendTransaction {
        from: utils::to_shardus_address(&from),
        to: utils::to_shardus_address(&to.to_string()),
        transaction_type: "friend".to_string(),
        alias: alias.clone(),
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn build_transfer_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    from: &LocalSigner<SigningKey>,
    to: &alloy::primitives::Address,
    amount: u128,
) -> TransferTransaction {
    let address = from.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let chat_id = {
        // lexically sort the two addresses, smaller address first
        let from_address = utils::to_shardus_address(&address);
        let to = utils::to_shardus_address(&to.to_string());

        let mut addresses = [from_address, to];
        addresses.sort();

        shardus_crypto
            .hash(&addresses.join("").into_bytes(), crypto::Format::Hex)
            .to_string()
    };

    let tx = serde_json::json!({
        "from": utils::to_shardus_address(&address),
        "to": utils::to_shardus_address(&to.to_string()),
        "amount": serde_json::json!({
            "dataType": "bi",
            "value": format!("{:x}",amount),
        }),
        "memo": "Liberdus Testing Framework Transaction",
        "chatId": chat_id,
        "type": "transfer",
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, from, &tx).expect("Failed to sign transaction");

    TransferTransaction {
        from: utils::to_shardus_address(&address),
        to: utils::to_shardus_address(&to.to_string()),
        amount: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: format!("{:x}", amount),
        },
        memo: Some("Liberdus Testing Framework Transaction".to_string()),
        chatId: chat_id,
        transaction_type: "transfer".to_string(),
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn build_deposite_stake_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    nominator: &LocalSigner<SigningKey>,
    nominee: &String,
    amount: u128,
) -> DepositStakeTransaction {
    let nominator_address = nominator.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let tx = serde_json::json!({
        "nominee": nominee,
        "stake": serde_json::json!({
            "dataType": "bi",
            "value": format!("{:x}",amount),
        }),
        "nominator": utils::to_shardus_address(&nominator_address),
        "type": "deposit_stake",
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, nominator, &tx).expect("Failed to sign transaction");

    DepositStakeTransaction {
        nominee: nominee.clone(),
        stake: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: format!("{:x}", amount),
        },
        nominator: utils::to_shardus_address(&nominator_address),
        transaction_type: "deposit_stake".to_string(),
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn build_register_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    alias: &String,
) -> RegisterTransaction {
    let address = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let alias_hash = shardus_crypto
        .hash(&alias.to_string().into_bytes(), crypto::Format::Hex)
        .to_string();

    let uncompressed_public_key = signer
        .credential()
        .verifying_key()
        .to_encoded_point(false)
        .to_string();

    let tx = serde_json::json!({
        "aliasHash": alias_hash,
        "from": utils::to_shardus_address(&address),
        "type": "register",
        "alias": &alias,
        "publicKey": uncompressed_public_key,
        "networkId": get_network_id(),
        "timestamp": now,
    });

    let signature =
        eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    RegisterTransaction {
        aliasHash: alias_hash,
        from: utils::to_shardus_address(&address),
        transaction_type: "register".to_string(),
        alias: alias.clone(),
        publicKey: uncompressed_public_key,
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn eth_sign_transaction(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    tx: &serde_json::Value,
) -> Option<ShardusSignature> {
    let from_address = signer.address().to_string();
    let message = shardus_crypto
        .hash(&tx.to_string().into_bytes(), crypto::Format::Hex)
        .to_string();
    let signature = signer
        .sign_message_sync(&message.clone().into_bytes())
        .expect("Failed to sign message");

    let parity_hex = match signature.v() {
        false => "1b",
        true => "1c",
    };

    match signature.to_k256() {
        Ok(k) => Some(ShardusSignature {
            owner: utils::to_shardus_address(&from_address),
            sig: format!("0x{}{}", k.to_string().to_lowercase(), parity_hex),
        }),
        Err(_e) => None,
    }
}

pub async fn inject_transaction(
    http_client: reqwest::Client,
    tx: &LiberdusTransactions,
    gateway_url: &String,
    verbosity: &bool,
) -> Result<InjectedTxResp, Box<dyn std::error::Error>> {
    let json_tx = match tx {
        LiberdusTransactions::Register(r) => {
            serde_json::to_value(r).expect("Failed to serialize transaction")
        }
        LiberdusTransactions::Transfer(t) => {
            serde_json::to_value(t).expect("Failed to serialize transaction")
        }
        LiberdusTransactions::Message(m) => {
            serde_json::to_value(m).expect("Failed to serialize transaction")
        }
        LiberdusTransactions::DepositStake(d) => {
            serde_json::to_value(d).expect("Failed to serialize transaction")
        }
        LiberdusTransactions::ChangeConfig(c) => {
            serde_json::to_value(c).expect("Failed to serialize transaction")
        }
    };

    let (payload, full_url) = {
        let payload = proxy::build_send_transaction_payload(&json_tx);

        (payload, &format!("{}/inject", gateway_url))
    };

    cli::verbose(verbosity, &format!("tx http payload {}", payload));

    let resp = match http_client.post(full_url).json(&payload).send().await {
        Ok(resp) => resp,
        Err(e) => {
            cli::verbose(verbosity, &format!("HTTP request failed: {}", e));
            return Err(e.into());
        }
    };

    // Get the raw response text for logging
    let response_text = resp.text().await?;
    cli::verbose(verbosity, &format!("raw response: {}", response_text));

    // Parse the response text as JSON
    match serde_json::from_str::<proxy::ProxyInjectedTxResp>(&response_text) {
        Ok(resp) => {
            if resp.result.is_some() && resp.error.is_none() {
                let result = resp
                    .result
                    .expect("Couldn't extract result from rpc response");
                
                cli::verbose(verbosity, &format!("tx injection result: {:?}", result));
                
                Ok(result)
            } else {
                cli::verbose(verbosity, &format!("tx injection failed - parsed resp: {:?}", resp));
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Tx Injection failed",
                )))
            }
        }
        Err(e) => {
            cli::verbose(verbosity, &format!("failed to parse response as JSON: {}", e));
            Err(e.into())
        }
    }
}
