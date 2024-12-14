use crate::{ crypto, transactions, rpc };
use alloy::signers::{
    k256::{ecdsa::SigningKey, Secp256k1}, local::LocalSigner, Error, SignerSync
};
use serde::{Serialize, Deserialize};

pub enum LiberdusTransactions {
    Register(RegisterTransaction),
    Transfer(TransferTransaction),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShardusSignature {
    owner: String,
    sig: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTransaction{
    aliasHash: String,
    from: String,
    #[serde(rename = "type")]
    transaction_type: String,
    alias: String,
    publicKey: String,
    timestamp: u128,
    sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferTransaction {
    from: String,
    to: String,
    amount: ShardusBigIntSerialized,
    #[serde(rename = "type")]
    transaction_type: String,
    timestamp: u128,
    sign: ShardusSignature,
}

#[derive(Serialize, Deserialize, Debug)]
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

    let signature = sign_transaction(shardus_crypto, from, &tx).expect("Failed to sign transaction");

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

    
    let signature = sign_transaction(shardus_crypto, signer, &tx).expect("Failed to sign transaction");

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

pub fn sign_transaction(shardus_crypto: &crypto::ShardusCrypto, signer: &LocalSigner<SigningKey>, tx: &serde_json::Value) -> Option<ShardusSignature> {
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

pub async fn inject_transaction(tx: &LiberdusTransactions) -> Result<rpc::RpcResponse, reqwest::Error> {

    let json_tx = match tx {
        LiberdusTransactions::Register(r) => {
            serde_json::to_value(r).expect("Failed to serialize transaction")
        },
        LiberdusTransactions::Transfer(t) => {
            serde_json::to_value(t).expect("Failed to serialize transaction")
        },
    };  

    let req = rpc::build_send_transaction_payload(&serde_json::to_value(&json_tx).expect("Failed to serialize transaction"));

    let resp = match reqwest::Client::new()
        .post("http://localhost:8545/")
        .json(&req)
        .send()
        .await {
            Ok(resp) => {
                resp.json::<rpc::RpcResponse>().await.expect("Failed to parse response")
            },
            Err(e) => {
                return Err(e);
            },
        };

    Ok(resp)

}