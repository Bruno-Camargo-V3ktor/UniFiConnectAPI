use rocket::{http::Status, response::status::Custom, serde::json::Json};

// Types
pub type Ok<B> = Custom<Json<B>>;
pub type Created<B> = Custom<Json<B>>;
pub type CustomStatus = Status;
pub type Accepted<B> = Custom<Json<B>>;

// Struct
pub struct Response;

// Impls
impl Response {
    pub fn new_custom<B>(status: u16, body: B) -> Custom<Json<B>> {
        Custom(Status { code: status }, Json(body))
    }

    pub fn new_ok<B>(body: B) -> Ok<B> {
        Self::new_custom(200, body)
    }

    pub fn new_created<B>(body: B) -> Created<B> {
        Self::new_custom(201, body)
    }

    pub fn new_custom_status(status: u16) -> CustomStatus {
        Status { code: status }
    }

    pub fn new_accepted<B>(body: B) -> Accepted<B> {
        Self::new_custom(202, body)
    }
}
