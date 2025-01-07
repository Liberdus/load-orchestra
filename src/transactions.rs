use crate::{ cli, proxy, crypto, load_injector::GatewayType, rpc};
use alloy::signers::{
    k256::ecdsa::SigningKey, local::LocalSigner, SignerSync
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct InjectedTxResp{
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepositStakeTransaction{
    pub nominee: String,
    pub stake: ShardusBigIntSerialized,
    pub nominator: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
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
pub struct RegisterTransaction{
    pub aliasHash: String,
    pub from: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub alias: String,
    pub publicKey: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriendTransaction{
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub alias: String,
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct MessageTransaction{
    pub from: String,
    pub to: String,
    pub amount: ShardusBigIntSerialized,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub chatId: String,
    pub message: String,
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
    pub timestamp: u128,
    pub sign: ShardusSignature,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShardusBigIntSerialized {
    dataType: String,
    value: String,
}

fn to_shardus_address(addr: &String) -> String {
    // cut 0x if it has it
    let mut address = addr.clone();
    if address.starts_with("0x") {
        address = address[2..].to_string();
    }

    // pad 00 until it become 64 characters
    while address.len() < 64 {
        address = format!("{}{}", address, "0");
    }

    address.to_lowercase()
}

pub fn build_message_transaction(
    shardus_crypto: &crypto::ShardusCrypto, 
    signer: &LocalSigner<SigningKey>, 
    to: &alloy::primitives::Address, 
    message: &String
) -> MessageTransaction {
    let from = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let chat_id = {
        // lexically sort the two addresses, smaller address first
        let mut joint_address: Vec<String> = Vec::new();
        joint_address.push(from.clone());
        joint_address.push(to.to_string().clone());
        joint_address.sort();
        shardus_crypto.hash(&joint_address.join("").into_bytes(), crypto::Format::Hex).to_string()
    };
    let tx = serde_json::json!({
        "from": to_shardus_address(&from),
        "amount": serde_json::json!({
            "dataType": "bi",
            "value": "1",
        }),
        "to": to_shardus_address(&to.to_string()),
        "type": "message",
        "chatId": chat_id,
        "message": message,
        "timestamp": now,
    });

    let signature = eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    MessageTransaction {
        amount: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: "1".to_string(),
        },
        from: to_shardus_address(&from),
        to: to_shardus_address(&to.to_string()),
        transaction_type: "message".to_string(),
        chatId: chat_id,
        message: message.clone(),
        timestamp: now,
        sign: signature,
    }

}

#[allow(dead_code)]
pub fn build_friend_transaction(
    shardus_crypto: &crypto::ShardusCrypto, 
    signer: &LocalSigner<SigningKey>, 
    to: &alloy::primitives::Address,
    alias: &String
) -> FriendTransaction {
    let from = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let tx = serde_json::json!({
        "from": to_shardus_address(&from),
        "to": to_shardus_address(&to.to_string()),
        "type": "friend",
        "alias": alias,
        "timestamp": now,
    });

    let signature = eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    FriendTransaction {
        from: to_shardus_address(&from),
        to: to_shardus_address(&to.to_string()),
        transaction_type: "friend".to_string(),
        alias: alias.clone(),
        timestamp: now,
        sign: signature,
    }

}


pub fn build_transfer_transaction(
    shardus_crypto: &crypto::ShardusCrypto, 
    from: &LocalSigner<SigningKey>, 
    to: &alloy::primitives::Address, 
    amount: u128
) -> TransferTransaction {

    let address = from.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let tx = serde_json::json!({
        "from": to_shardus_address(&address),
        "to": to_shardus_address(&to.to_string()),
        "amount": serde_json::json!({
            "dataType": "bi",
            "value": format!("{:x}",amount),
        }),
        "type": "transfer",
        "timestamp": now,
    }); 

    let signature = eth_sign_transaction(shardus_crypto, from, &tx).expect("Failed to sign transaction");

    TransferTransaction {
        from: to_shardus_address(&address),
        to: to_shardus_address(&to.to_string()),
        amount: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: format!("{:x}", amount),
        },
        transaction_type: "transfer".to_string(),
        timestamp: now,
        sign: signature,
    }
}

pub fn build_deposite_stake_transaction(
    shardus_crypto: &crypto::ShardusCrypto, 
    nominator: &LocalSigner<SigningKey>, 
    nominee: &String, 
    amount: u128
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
        "nominator": to_shardus_address(&nominator_address),
        "type": "deposit_stake",
        "timestamp": now,
    }); 

    let signature = eth_sign_transaction(shardus_crypto, nominator, &tx).expect("Failed to sign transaction");

    DepositStakeTransaction {
        nominee: nominee.clone(),
        stake: ShardusBigIntSerialized {
            dataType: "bi".to_string(),
            value: format!("{:x}", amount),
        },
        nominator: to_shardus_address(&nominator_address),
        transaction_type: "deposit_stake".to_string(),
        timestamp: now,
        sign: signature,
    }
}


pub fn build_register_transaction(shardus_crypto: &crypto::ShardusCrypto, signer: &LocalSigner<SigningKey>, alias: &String) -> RegisterTransaction {
    let address = signer.address().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let alias_hash = shardus_crypto.hash(&alias.to_string().into_bytes(), crypto::Format::Hex).to_string();

    let uncompressed_public_key = signer.credential().verifying_key().to_encoded_point(false).to_string();

    let tx = serde_json::json!({
        "aliasHash": alias_hash,
        "from": to_shardus_address(&address),
        "type": "register",
        "alias": &alias,
        "publicKey": uncompressed_public_key,
        "timestamp": now,
    }); 

    
    let signature = eth_sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

    RegisterTransaction {
        aliasHash: alias_hash,
        from: to_shardus_address(&address),
        transaction_type: "register".to_string(),
        alias: alias.clone(),
        publicKey: uncompressed_public_key,
        timestamp: now,
        sign: signature,
    }
}

pub fn eth_sign_transaction(shardus_crypto: &crypto::ShardusCrypto, signer: &LocalSigner<SigningKey>, tx: &serde_json::Value) -> Option<ShardusSignature> {
    let from_address = signer.address().to_string();
    let message = shardus_crypto.hash(&tx.to_string().into_bytes(), crypto::Format::Hex).to_string();
    let signature = signer.sign_message_sync(&message.clone().into_bytes()).expect("Failed to sign message");

    let parity_hex = match signature.v() {
        false => "1b",
        true => "1c",
    };

    let serialized_signature = match signature.to_k256() {
        Ok(k) => {
            Some( ShardusSignature {
                owner: to_shardus_address(&from_address),
                sig: format!("0x{}{}", k.to_string().to_lowercase(), parity_hex),
            })
        },
        Err(_e) => {
            None
        },
    };

    serialized_signature
}

pub async fn inject_transaction(http_client: reqwest::Client, tx: &LiberdusTransactions, gateway_type: &GatewayType, gateway_url: &String, verbosity: &bool) -> Result<InjectedTxResp, Box<dyn std::error::Error>> {

    let json_tx = match tx {
        LiberdusTransactions::Register(r) => {
            serde_json::to_value(r).expect("Failed to serialize transaction")
        },
        LiberdusTransactions::Transfer(t) => {
            serde_json::to_value(t).expect("Failed to serialize transaction")
        },
        LiberdusTransactions::Message(m) => {
            serde_json::to_value(m).expect("Failed to serialize transaction")
        },
        LiberdusTransactions::DepositStake(d) => {
            serde_json::to_value(d).expect("Failed to serialize transaction")
        },
    };  

    let (payload, full_url) = match gateway_type {
        GatewayType::Rpc => {
            let payload = rpc::build_send_transaction_payload(&json_tx);
            let url = gateway_url;

            (payload, url)
        },
        GatewayType::Proxy => {
            let payload = proxy::build_send_transaction_payload(&json_tx);

            (payload, &format!("{}/inject", gateway_url))
        },

    };

    cli::verbose(verbosity, &format!("tx http payload {}", payload));

    let resp = match http_client
        .post(full_url)
        .json(&payload)
        .send()
        .await {
            Ok(resp) => {
                resp
            },
            Err(e) => {
                return Err(e.into());
            },
        };

    match gateway_type {
        GatewayType::Rpc => {
            match resp.json::<rpc::RpcResponse>().await {
                Ok(resp) => {
                    if resp.result.is_some() {
                        return Ok(resp.result.expect("Couldn't extract result from rpc response"));
                    }
                    else if resp.error.is_some() {
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, resp.error.expect("Couldn't extract error from rpc response").message)));
                    }
                    else{
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "RPC internal Failure")));
                    }
                },
                Err(e) => {
                    return Err(e.into());
                },
            }
            
        },
        GatewayType::Proxy => {
            match resp.json::<proxy::ProxyInjectedTxResp>().await {
                Ok(resp) => {
                    if resp.result.is_some() && resp.error.is_none() {
                        return Ok(resp.result.expect("Couldn't extract result from rpc response"));
                    }else{
                        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Tx Injection failed")));
                    }
                },
                Err(e) => {
                    return Err(e.into());
                },
            }
        },
    }

}
