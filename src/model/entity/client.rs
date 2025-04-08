use std::collections::HashMap;

use crate::{configurations::config::ClientsConfig, db::mongo_db::serde_object_id, utils::validator::Validator};
use chrono::{DateTime, Local};
use rocket::serde::{Deserialize, Serialize};

use super::Entity;

// Enums
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ClientStatus {
    Approved,
    Pending,
    Reject,
    Expired,
}

// Structs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientData {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub approver_code: Option<String>,
    pub fields: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub id: Option<String>,
    pub data: Option<ClientData>,
    pub mac: String,
    pub site: String,
    pub minutes: u16,
    pub connect: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,

    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub fields: HashMap<String, String>,

    pub mac: String,
    pub site: String,
    pub status: ClientStatus,

    pub hostname: Option<String>,
    pub tx_bytes: Option<usize>,
    pub rx_bytes: Option<usize>,

    pub time_connection: String,
    pub start_time: DateTime<Local>,
    pub approver: String,
}

// Impls
#[allow(unused)]
impl Client {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            full_name: String::from("---"),
            email: String::from("---"),
            phone: String::from("---"),
            fields: HashMap::new(),

            mac: String::from("---"),
            site: String::from("---"),
            
            status: ClientStatus::Pending,
            hostname: None,
            tx_bytes: None,
            rx_bytes: None,
            time_connection: String::from("0"),
            start_time: Local::now(),
            approver: String::from("---"),
        }
    }

    pub fn new_with_data(data: &ClientData) -> Self {
        Self {
            id: String::new(),
            full_name: data.full_name.clone(),
            email: data.email.clone(),
            phone: data.phone.clone(),
            fields: data.fields.clone(),

            mac: String::from("---"),
            site: String::from("---"),
            status: ClientStatus::Pending,

            hostname: None,
            tx_bytes: None,
            rx_bytes: None,
            time_connection: String::from("0"),
            start_time: Local::now(),
            approver: String::from("---") 
        }
    }

    pub fn new_with_info(info: &ClientInfo) -> Self {
        let mut client = Self {
            id: String::new(),
            full_name: String::from("---"),
            email: String::from("---"),
            phone: String::from("---"),
            fields: HashMap::new(),

            mac: info.mac.clone(),
            site: info.site.clone(),
            
            status: if info.connect {
                ClientStatus::Approved
            } else {
                ClientStatus::Reject
            },
            hostname: None,
            tx_bytes: None,
            rx_bytes: None,
            time_connection: format!("{}", info.minutes.clone()),
            start_time: Local::now(),
            approver: String::from("---"),
        };

        if let Some(data) = info.data.clone() {
            client.full_name = data.full_name;
            client.email = data.email;
            client.phone = data.phone;
            client.fields = data.fields;
        }

        client
    }
    
}

impl ClientData {
    pub fn validate_form(&self, config: ClientsConfig) -> bool {
        Validator::validate_client(&config, &self)
    }
}

// Impls
impl Entity<String> for Client {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, new_id: String) {
        self.id = new_id;
    }

    fn get_name() -> String {
        String::from("Clients")
    }
}  
