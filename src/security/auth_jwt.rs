use rocket::serde::{Deserialize, Serialize};
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result};

// Struct
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// Functions
pub fn create_token(user_id: &String) -> String {
    let key = env::var("ROCKET_SECRET_KEY").unwrap();

    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60 * 60);

    let content = Claims {
        sub: user_id.clone(),
        exp: expiration.as_secs() as usize,
    };

    encode(
        &Header::default(),
        &content,
        &EncodingKey::from_secret(key.as_ref()),
    )
    .unwrap()
}

pub fn validate_token(token: String) -> Result<Claims> {
    let key = env::var("ROCKET_SECRET_KEY").unwrap();

    decode(
        token.as_str(),
        &DecodingKey::from_secret(key.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
