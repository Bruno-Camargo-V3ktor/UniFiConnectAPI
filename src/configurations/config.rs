
use std::fs::File;
use std::io::prelude::*;
use rocket::{config::SecretKey, figment::Figment};
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;

// Types
pub type ConfigApp = RwLock<ConfigApplication>;

// Structs
#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,     
    pub port: u16,                   
    pub workers: usize,                 
    pub log_level: String,             
    pub keep_alive: u32,         
    pub secret_key: String,
    pub files_dir: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UnifiConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientGroup {
    pub name: String,
    pub time_conneciton: usize,
    pub permissions: Vec<String>,
    pub restrictions: Vec<String>,
    pub upload_limit: usize,
    pub download_limit: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientsConfig {
    pub groups: Vec<ClientGroup>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApproversConfig {
    pub code_size: usize,
    pub validity_days_code: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminsConfig {
    pub token_expirantion: usize,
} 

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn to_rocket_config(&self) -> rocket::figment::Figment {
        let config = rocket::Config {
            address: self.server.address.parse().unwrap(),
            port: self.server.port.clone(),
            workers: self.server.workers,                 
            log_level: self.server.log_level.parse().unwrap(),  
            keep_alive: self.server.keep_alive.clone(),  
            secret_key: SecretKey::from(self.server.secret_key.as_bytes()),
            ..rocket::Config::default()
        };

        let database_url = self.database.get_formated_url();

        let figment = Figment::from(config).merge(("databases.mongodb", rocket_db_pools::Config {
            url: database_url,
            min_connections: Some(64),
            max_connections: 1024,
            connect_timeout: 5,
            idle_timeout: Some(120),
            extensions: None,
            ..rocket_db_pools::Config::default()
        }));

        figment
    }

}

impl DatabaseConfig {
    pub fn get_formated_url(&self) -> String {
        self.url.clone()
            .replacen("{}", &self.username.clone(), 1)
            .replacen("{}", &self.password.clone(), 1)
    }
}