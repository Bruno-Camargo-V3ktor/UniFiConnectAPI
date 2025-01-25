use chrono::Local;
use rocket::{
    response::status::Custom,
    serde::{Deserialize, Serialize, json::Json},
};

// Types
pub type CustomError = Custom<Json<Error>>;
pub type Unauthorized = Custom<Json<Error>>;
pub type BadRequest = Custom<Json<Error>>;
pub type NotFound = Custom<Json<Error>>;

// Structs
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub err: String,
    pub time: String,
    pub status: u16,
}

// Impls
impl Error {
    pub fn new_with_custom(msg: &str, time: String, status: u16) -> CustomError {
        let error = Error {
            err: msg.to_string(),
            time,
            status,
        };

        Custom(rocket::http::Status { code: status }, Json(error))
    }

    pub fn new_unauthorized(msg: &str) -> Unauthorized {
        Self::new_with_custom(msg, Local::now().to_string(), 401)
    }

    pub fn new_bad_request(msg: &str) -> BadRequest {
        Self::new_with_custom(msg, Local::now().to_string(), 400)
    }

    pub fn new_not_found(msg: &str) -> NotFound {
        Self::new_with_custom(msg, Local::now().to_string(), 404)
    }
}
