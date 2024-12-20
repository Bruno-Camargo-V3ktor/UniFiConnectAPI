use rocket::serde::{Deserialize, Serialize};


// Structs
#[ derive( Serialize, Deserialize ) ]
pub struct Error {
    err: String,
    time: String,
    status: u16
}
