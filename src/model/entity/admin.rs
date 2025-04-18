use crate::{db::mongo_db::serde_object_id, ldap::ldap::LdapUser};
use rocket::serde::{Deserialize, Serialize};

use super::Entity;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminToken {
    pub token: String
}

// Impls
impl Admin {
    pub fn new_with_ldap_user(ldpa_user: &LdapUser) -> Self {
        Self {
            id: String::new(),
            name: ldpa_user.name.clone(),
            username: ldpa_user.username.clone(),
            password: None
        }
    }
}


impl Entity<String> for Admin {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, new_id: String) {
        self.id = new_id;
    }

    fn get_name() -> String {
        String::from("Admins")
    }
}  
