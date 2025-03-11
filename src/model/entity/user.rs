use super::{Entity, client::ClientData};
use crate::{db::mongo_db::serde_object_id, ldap::ldap::LdapUser};
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

impl User {
    pub fn new_with_ldap_user(ldap_user: &LdapUser) -> Self {
        let data = ClientData {
            full_name: ldap_user.name.clone(),
            companion: String::new(),
            email: ldap_user.email.clone(),
            client_type: String::new(),
            phone: String::new(),
            cpf: None,
            menssage: None,
            approver_code: None
        };

        Self {
            id: String::new(),
            username: ldap_user.username.clone(),
            email: ldap_user.email.clone(),
            password: String::new(),
            data
        }
    }
}