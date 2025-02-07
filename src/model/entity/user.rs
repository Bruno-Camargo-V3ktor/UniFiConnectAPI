use super::{Entity, client::ClientData};
use crate::db::mongo_db::serde_object_id;
use serde::{Deserialize, Serialize};

// Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id", with = "serde_object_id")]
    id: String,
    username: String,
    password: String,
    email: String,
    group: String,
    data: ClientData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    username: String,
    password: String,
    group: Option<String>,
    approver_code: Option<String>,
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
