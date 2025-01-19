use rocket::serde::{Deserialize, Serialize};

// Structs
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub err: String,
    pub time: String,
    pub status: u16,
}
