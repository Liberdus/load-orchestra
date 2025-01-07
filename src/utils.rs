use std::io::Write;
use rand::{self, Rng};

pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    // Generate a random string by selecting random characters from the CHARSET
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len()); // Generate a random index
            CHARSET[idx] as char
        })
        .collect()
}

pub struct InjectionStats{
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

pub fn stdout_injection_stats(stats: &InjectionStats, verbosity: &bool) {
    if *verbosity {
        return;
    }
    let failure_rates = (stats.failed as f64 / stats.total as f64) * 100.0;
    print!(
        "\rTotal: {:<10} Success: {:<10} Failed: {:<10} Failure: {:<10.2}%",
        stats.total, stats.success, stats.failed, failure_rates
    );
    std::io::stdout().flush().unwrap(); 
}

pub fn stdout_register_progress(max: usize, progress: usize) {
    let percentage = (progress as f64 / max as f64) * 100.0;
    print!(
        "\rRegistering {:?} / {:?} Wallets. ({:<.2}%)",
        progress, max, percentage
    );
    std::io::stdout().flush().unwrap(); 
}

pub fn append_json_to_file(file_path: &str, json_value: &serde_json::Value) -> std::io::Result<()> {
    let path = std::path::Path::new(file_path);

    // Ensure the parent directories exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?; // Creates all directories in the path
    }
    let file = std::fs::OpenOptions::new()
        .create(true)   
        .append(true)   
        .open(file_path)?;

    let mut writer = std::io::BufWriter::new(file);

    let json_string = serde_json::to_string(json_value)?;

    writeln!(writer, "{}", json_string)?;

    Ok(())
}


/// check if it's a valid 32 byte hex string, 0x prefix is optional
#[allow(dead_code)]
pub fn is_valid_shardus_address(address: &str) -> bool {
    let address = address.trim_start_matches("0x");
    if address.len() != 64 {
        return false;
    }
    address.chars().all(|c| c.is_ascii_hexdigit())
}
