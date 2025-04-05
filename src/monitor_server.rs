#![allow(dead_code)]
use std::collections::HashMap;

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub struct MonitorApiReportResp {
    pub nodes: MonitorApiNodelist,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub struct MonitorApiNodelist {
    joining: HashMap<String, serde_json::Value>,
    active: HashMap<String, serde_json::Value>,
    syncing: HashMap<String, serde_json::Value>,
    standby: HashMap<String, serde_json::Value>,
}

pub async fn collect_joining(monitor_server_url: &str) -> Vec<String> {
    let client = reqwest::Client::new();
    let full_url = format!("{}/api/report", monitor_server_url);
    let resp = match client.get(full_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to connect to monitor server: {}", e);
            std::process::exit(1);
        }
    };
    let nominees = match resp.json::<MonitorApiReportResp>().await {
        Ok(nominees) => nominees,
        Err(e) => {
            eprintln!("Failed to parse monitor server response: {}", e);
            std::process::exit(1);
        }
    };

    let mut joining = Vec::new();

    for (id, _) in nominees.nodes.joining.iter() {
        joining.push(id.to_string());
    }

    joining
}

pub async fn collect_active(monitor_server_url: &str) -> Vec<String> {
    let client = reqwest::Client::new();
    let full_url = format!("{}/api/report", monitor_server_url);
    let resp = match client.get(full_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to connect to monitor server: {}", e);
            std::process::exit(1);
        }
    };
    let nominees = match resp.json::<MonitorApiReportResp>().await {
        Ok(nominees) => nominees,
        Err(e) => {
            eprintln!("Failed to parse monitor server response: {}", e);
            std::process::exit(1);
        }
    };

    let mut joining = Vec::new();

    for (id, _) in nominees.nodes.active.iter() {
        joining.push(id.to_string());
    }

    joining
}

pub async fn collect_all(monitor_server_url: &str) -> Vec<String> {
    let mut joining = collect_joining(monitor_server_url).await;
    let active = collect_active(monitor_server_url).await;

    joining.extend(active);

    joining
}
