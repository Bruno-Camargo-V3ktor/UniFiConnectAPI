use rocket::{
    response::status::Custom,
    serde::{Deserialize, Serialize, json::Json},
};

// Structs
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub err: String,
    pub time: String,
    pub status: u16,
}

// Impls
impl Error {
    pub fn new_with_custom(msg: &str, time: String, status: u16) -> Custom<Json<Self>> {
        let error = Error {
            err: msg.to_string(),
            time,
            status,
        };

        Custom(rocket::http::Status { code: status }, Json(error))
    }
}
