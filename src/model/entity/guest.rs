use crate::db::mongo_db::serde_object_id;
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
    pub oui: Option<String>,

    pub time_connection: String,
    pub start_time: DateTime<Local>,
    pub approver: String,
}
