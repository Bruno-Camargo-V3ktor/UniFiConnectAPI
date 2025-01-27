use crate::db::mongo_db::serde_object_id;
use chrono::{DateTime, Local};
use rocket::serde::{Deserialize, Serialize};

// Struct
#[derive(Debug, Deserialize, Serialize)]
pub struct Approver {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub secrete_code: String,
    pub validity: Option<DateTime<Local>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApproverData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub secrete_code: String,
}
