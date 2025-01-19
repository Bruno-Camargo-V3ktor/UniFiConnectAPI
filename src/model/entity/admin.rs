use crate::db::mongo_db::serde_object_id;
use rocket::serde::{Deserialize, Serialize};

// Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct AdminLogin {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Admin {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,
    pub name: String,
    pub username: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminData {
    pub name: String,
    pub username: String,
    pub password: Option<String>,
}
