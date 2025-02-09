use super::{Entity, client::ClientData};
use crate::db::mongo_db::serde_object_id;
use serde::{Deserialize, Serialize};

// Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub data: ClientData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
    pub group: Option<String>,
    pub approver_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    pub password: Option<String>,
    pub email: Option<String>,
    pub data: Option<ClientData>,
}

// Impls
impl Entity<String> for User {
    fn get_name() -> String {
        "Users".to_string()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, new_id: String) {
        self.id = new_id
    }
}
