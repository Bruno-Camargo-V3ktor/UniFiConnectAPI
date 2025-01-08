use rocket::serde::{Deserialize, Serialize};

// Enums
#[ derive(Deserialize, Debug) ]
//#[ serde(tag = "type") ]
#[ serde(untagged) ]
pub enum GuestData {
    Info(GuestInfo),
    Form(GuestForm)
}

// Structs
#[ derive( Serialize, Deserialize, Debug ) ]
pub struct GuestForm {
    pub au_code: Option<u16>
}

#[ derive( Serialize, Deserialize, Debug ) ]
pub struct GuestInfo {
    pub id: Option<String>,
    pub mac: String,
    pub site: String,
    pub minutes: u16,
}

#[ derive( Serialize, Deserialize ) ]
pub struct Guest {

}
