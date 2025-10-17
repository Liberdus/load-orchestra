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
    pub xmessage: XMessage,
    pub fee: ShardusBigIntSerialized,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XMessage {
    pub encrypted: bool,
    #[serde(rename = "encryptionMethod")]
    pub encryption_method: String,
    pub message: String,
    pub pqSharedKey: String,
    #[serde(rename = "senderInfo")]
    pub sender_info: String,
    #[serde(rename = "sent_timestamp")]
    pub sent_timestamp: u64,
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
    let XMESSAGE: XMessage = XMessage {
        encrypted: true,
        encryption_method: "xchacha20poly1305".to_string(),
        message: "zknTZuSmFP0clZVqmH2VQS/+R5Ezotgb5L9cJ6WHdy/1iQGPoU1AbqoVJk1gL6GfHc74OlB5/o/n4O8p55DqTRwUMJL73k+GMaXHPlI6Nch3NJg=".to_string(),
        sender_info: "test_sender_info".to_string(),
        sent_timestamp: 1,
        pqSharedKey: "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".to_string(),
    };
    let tx = serde_json::json!({
        "from": utils::to_shardus_address(&from),
        "amount": serde_json::json!({
            "dataType": "bi",
            "value": "0",
        }),
        "to": utils::to_shardus_address(&to.to_string()),
        "type": "message",
        "chatId": chat_id,
        "message": message,
        "networkId": get_network_id(),
        "timestamp": now,
        "xmessage": XMESSAGE,
         "fee": serde_json::json!({
            "dataType": "bi",
            "value": "769200000000",
        }),
    });

    let signature =
        eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    // Manually construct the MessageTransaction struct, just like register transaction does
    MessageTransaction {
        from: utils::to_shardus_address(&from),
        to: utils::to_shardus_address(&to.to_string()),
        amount: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: "0".to_string(),
        },
        transaction_type: "message".to_string(),
        chatId: chat_id,
        message: message.clone(),
        networkId: get_network_id().to_string(),
        timestamp: now,
        sign: signature,
        xmessage: XMESSAGE,
        fee: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: "769200000000".to_string(),
        },
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

pub fn eth_sign_transaction_with_string(
    shardus_crypto: &crypto::ShardusCrypto,
    signer: &LocalSigner<SigningKey>,
    tx_json_string: &str,
) -> Option<ShardusSignature> {
    let from_address = signer.address().to_string();
    let message = shardus_crypto
        .hash(&tx_json_string.as_bytes().to_vec(), crypto::Format::Hex)
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

pub fn eth_verify_signature(
    shardus_crypto: &crypto::ShardusCrypto,
    tx: &serde_json::Value,
    signature: &ShardusSignature,
    expected_address: &alloy::primitives::Address,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    use alloy::primitives::PrimitiveSignature;

    // Hash the transaction the same way as in eth_sign_transaction
    let message_hash = shardus_crypto
        .hash(&tx.to_string().into_bytes(), crypto::Format::Hex)
        .to_string();

    // Parse the signature from the ShardusSignature
    let sig_str = &signature.sig;
    if !sig_str.starts_with("0x") || sig_str.len() != 132 {
        return Err("Invalid signature format".into());
    }

    // Remove the "0x" prefix
    let sig_hex = &sig_str[2..];
    
    // The signature is in format: r(64) + s(64) + v(2)
    // Extract r, s, and v components
    let r_hex = &sig_hex[0..64];
    let s_hex = &sig_hex[64..128];
    let v_hex = &sig_hex[128..130];

    // Parse the signature components
    let r_bytes = hex::decode(r_hex)?;
    let s_bytes = hex::decode(s_hex)?;
    let v_byte = u8::from_str_radix(v_hex, 16)?;
    
    // Convert to recovery ID (v - 27 for legacy signatures, but here it's already adjusted)
    let recovery_id = if v_byte == 0x1b { false } else if v_byte == 0x1c { true } else {
        return Err("Invalid recovery ID".into());
    };

    // Create the signature from components
    let mut sig_bytes = [0u8; 64];
    if r_bytes.len() != 32 || s_bytes.len() != 32 {
        return Err("Invalid signature component length".into());
    }
    sig_bytes[0..32].copy_from_slice(&r_bytes);
    sig_bytes[32..64].copy_from_slice(&s_bytes);
    
    let signature = PrimitiveSignature::from_bytes_and_parity(&sig_bytes, recovery_id);

    // Verify the signature by recovering the address
    let recovered_address = signature.recover_address_from_msg(&message_hash.as_bytes())?;

    Ok(recovered_address == *expected_address)
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::signers::local::LocalSigner;
    
    #[test]
    fn test_eth_sign_and_verify() {
        let shardus_crypto = crypto::ShardusCrypto::new(
            "69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc"
        );
        
        // Create a test signer
        let signer = LocalSigner::random();
        let expected_address = signer.address();
        
        // Create a test transaction
        let tx = serde_json::json!({
            "from": utils::to_shardus_address(&expected_address.to_string()),
            "to": "test_to_address",
            "amount": {
                "dataType": "bi",
                "value": "1000"
            },
            "type": "test",
            "timestamp": 1234567890_u128,
        });
        
        // Sign the transaction
        let signature = eth_sign_transaction(&shardus_crypto, &signer, &tx)
            .expect("Failed to sign transaction");
        
        // Verify the signature
        let is_valid = eth_verify_signature(&shardus_crypto, &tx, &signature, &expected_address)
            .expect("Failed to verify signature");
        
        assert!(is_valid, "Signature verification should succeed");
        
        // Test with wrong address (should fail)
        let wrong_signer = LocalSigner::random();
        let wrong_address = wrong_signer.address();
        
        let is_invalid = eth_verify_signature(&shardus_crypto, &tx, &signature, &wrong_address)
            .expect("Failed to verify signature with wrong address");
        
        assert!(!is_invalid, "Signature verification should fail with wrong address");
    }
    
    #[test]
    fn test_message_transaction_json_structure() {
        let shardus_crypto = crypto::ShardusCrypto::new(
            "69fa4195670576c0160d660c3be36556ff8d504725be8a59b5a96509e0c994bc"
        );
        
        let signer = LocalSigner::random();
        let to_address = alloy::primitives::Address::repeat_byte(0x42);
        let message = "test message".to_string();
        
        // Build the transaction
        let message_tx = build_message_transaction(&shardus_crypto, &signer, &to_address, &message);
        
        // Serialize to JSON to see the actual structure
        let json_output = serde_json::to_string_pretty(&message_tx).unwrap();
        println!("Message Transaction JSON:\n{}", json_output);
        
        // Also test what the original tx object looked like before signing
        let from = signer.address().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let chat_id = {
            let from_address = utils::to_shardus_address(&from);
            let to = utils::to_shardus_address(&to_address.to_string());
            let mut addresses = [from_address, to];
            addresses.sort();
            shardus_crypto
                .hash(&addresses.join("").into_bytes(), crypto::Format::Hex)
                .to_string()
        };
        
        let original_tx = serde_json::json!({
            "from": utils::to_shardus_address(&from),
            "amount": serde_json::json!({
                "dataType": "bi",
                "value": "0",
            }),
            "to": utils::to_shardus_address(&to_address.to_string()),
            "type": "message",
            "chatId": chat_id,
            "message": message,
            "networkId": get_network_id(),
            "timestamp": now,
            "xmessage": {
                "encrypted": true,
                "encryptionMethod": "xchacha20poly1305",
                "message": "test_encrypted_message",
            },
            "fee": {
                "dataType": "bi",
                "value": "B317E48C00"
            },
        });
        
        let original_json = serde_json::to_string_pretty(&original_tx).unwrap();
        println!("Original TX JSON (what gets signed):\n{}", original_json);
    }
}
