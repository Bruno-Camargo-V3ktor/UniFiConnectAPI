use std::env;

use crate::db::mongo_db::serde_object_id;
use chrono::{DateTime, Duration, Local, TimeZone};
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
    pub id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub secrete_code: Option<String>,
}

// Impls
impl Approver {
    pub fn create_validity(&mut self) {
        let days = env::var("VALIDITY_DAYS_APPROVAL_CODE")
            .expect("VALIDITY_DAYS_APPROVAL_CODE NOT DEFINED")
            .parse::<i64>()
            .expect("VALIDITY_DAYS_APPROVAL_CODE NOT NUMBER");

        if days <= 0 { return; }

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
