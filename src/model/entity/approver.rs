use crate::db::mongo_db::serde_object_id;
use chrono::{DateTime, Duration, Local, TimeZone};
use rocket::serde::{Deserialize, Serialize};

use super::Entity;

// Struct
#[derive(Debug, Deserialize, Serialize)]
pub struct Approver {
    #[serde(rename = "_id", with = "serde_object_id")]
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub secrete_code: String,
    pub approved_types: Vec<String>,
    pub validity: Option<DateTime<Local>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApproverData {
    pub username: String,
    pub email: String,
    pub password: String,
    pub secrete_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApproverLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApproverCode {
    pub new_code: String,
    pub days: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApproverUpdate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub approved_types: Option<Vec<String>>,
    pub secrete_code: Option<String>,
}

// Impls
impl Entity<String> for Approver {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn set_id(&mut self, new_id: String) {
        self.id = new_id;
    }

    fn get_name() -> String {
        String::from("Approvers")
    }
}

impl Approver {
    pub fn create_validity(&mut self, days: i64) {
        if days <= 0 {
            return;
        }

        let date = Local::now()
            .checked_add_signed(Duration::days(days))
            .expect("Error creating expiration date")
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let validity_date = Local.from_local_datetime(&date).unwrap();

        self.validity = Some(validity_date);
    }
}

impl ApproverCode {
    pub fn new(code: String, days: usize) -> Self {
        Self {
            new_code: code,
            days: Some(days),
        }
    }
}
