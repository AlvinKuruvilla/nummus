use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::fs::read_dir;
use std::io;
use std::path::PathBuf;
/// Get the project root (relative to closest Cargo.lock file)
// adapted from https://docs.rs/project-root/latest/project_root/fn.get_project_root.html
pub fn get_project_root() -> io::Result<String> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = read_dir(p)?
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name() == "Cargo.lock");
        if has_cargo {
            let root = PathBuf::from(p).to_str().unwrap().to_owned();
            return Ok(root);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Ran out of places to find Cargo.lock",
    ))
}

pub static RUN_CONFIG: Lazy<RunConfig> = Lazy::new(RunConfig::create);
#[derive(Serialize, Deserialize)]
pub struct RunConfig {
    pub org_count: usize,
    pub transaction_count: usize,
    pub addresses_per_organization: usize,
}
impl RunConfig {
    pub fn create() -> Self {
        let file_data = fs::read_to_string(get_project_root().unwrap() + "/" + "run_config.json")
            .expect("Couldn't find or load config file.");
        let config_data: RunConfig = serde_json::from_str(&file_data).unwrap();
        RunConfig {
            org_count: config_data.org_count,
            transaction_count: config_data.transaction_count,
            addresses_per_organization: config_data.addresses_per_organization,
        }
    }
}
impl fmt::Display for RunConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "Org Count: {}", self.org_count);
        let _ = write!(f, "Transaction Count: {}", self.transaction_count);
        write!(
            f,
            "Addresses per Organization: {}",
            self.addresses_per_organization
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        println!("Org Count: {}", RUN_CONFIG.org_count);
        println!("Transaction Count: {}", RUN_CONFIG.transaction_count);
        println!(
            "Addresses per Organization: {}",
            RUN_CONFIG.addresses_per_organization
        )
    }
}
