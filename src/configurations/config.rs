
use std::fs::File;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};

// Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub address: String,     
    pub port: usize,                   
    pub workers: usize,                 
    pub log_level: String,             
    pub keep_alive: usize,         
    pub secret_key: String 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnifiConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientGroup {
    pub name: String,
    pub time_conneciton: usize,
    pub permissions: Vec<String>,
    pub restrictions: Vec<String>,
    pub upload_limit: usize,
    pub download_limit: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientsConfig {
    pub groups: Vec<ClientGroup>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApproversConfig {
    pub code_size: usize,
    pub validity_days_code: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminsConfig {
    pub token_expirantion: usize,
} 

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigApplication {
    pub server: ServerConfig,
    pub unifi: UnifiConfig,
    pub database: DatabaseConfig,
    pub clients: ClientsConfig,
    pub approvers: ApproversConfig,
    pub admins: AdminsConfig
}

// Impls
impl ConfigApplication {
    pub fn new() -> Self {
        let mut file = File::open(".config.json").expect("Configuration file not found");
        let mut json_str = String::new();
        file.read_to_string(&mut json_str).expect("Error reading configuration file");

        let config: Self = serde_json::from_str(&json_str).expect("The settings are incorrect");
        config
    }

    pub fn save(&self) {
        let json_str = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(".config.json").expect("Configuration file not found");
        file.write(json_str.as_bytes()).expect("Error saving settings file");
    }

}