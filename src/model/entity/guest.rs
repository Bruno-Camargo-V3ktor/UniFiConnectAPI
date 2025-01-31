use crate::{db::mongo_db::serde_object_id, utils::validator::Validator};
use chrono::{DateTime, Local};
use rocket::serde::{Deserialize, Serialize};

// Enums
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GuestData {
    Info(GuestInfo),
    Form(GuestForm),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum GuestStatus {
    Approved,
    Pending,
    Reject,
    Expired,
}

// Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct GuestForm {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub cpf: String,
    pub au_code: Option<String>,
    pub menssage: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuestInfo {
    pub id: Option<String>,
    pub mac: String,
    pub site: String,
    pub minutes: u16,
    pub approved: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Guest {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,

    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub cpf: String,

    pub mac: String,
    pub site: String,
    pub status: GuestStatus,

    pub hostname: Option<String>,
    pub tx_bytes: Option<usize>,
    pub rx_bytes: Option<usize>,

    pub time_connection: String,
    pub start_time: DateTime<Local>,
    pub approver: String,
}

// Impls
impl Guest {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            full_name: String::from("---"),
            email: String::from("---"),
            phone: String::from("---"),
            cpf: String::from("---"),
            mac: String::from("---"),
            site: String::from("---"),
            status: GuestStatus::Pending,
            hostname: None,
            tx_bytes: None,
            rx_bytes: None,
            time_connection: String::from("0"),
            start_time: Local::now(),
            approver: String::from("---"),
        }
    }
}

impl GuestForm {
    pub fn validate_form(&self) -> bool {
        Validator::validate_phone(&self.phone)
            && Validator::validate_email(&self.email)
            && Validator::validate_cpf(&self.cpf)
    }
}
