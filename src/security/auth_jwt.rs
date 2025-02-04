use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    serde::{Deserialize, Serialize},
};
use rocket_db_pools::Connection;
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result};

use crate::{
    db::mongo_db::MongoDb,
    model::{
        entity::admin::Admin,
        repository::{Repository, mongo_repository::MongoRepository},
    },
};

// Struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

// Functions
pub fn create_token(user_id: &String) -> String {
    let key = env::var("ROCKET_SECRET_KEY").unwrap();
    let hours = env::var("TOKEN_EXPERIMENT_TIME")
        .unwrap_or("1".to_string())
        .parse::<u64>()
        .unwrap()
        * 60;

    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60 * hours);

    let content = Claims {
        sub: user_id.clone(),
        exp: expiration.as_secs() as usize,
    };

    encode(
        &Header::default(),
        &content,
        &EncodingKey::from_secret(key.as_bytes()),
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

// Guards
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();

        if keys.len() < 1 {
            return Outcome::Forward(Status::Continue);
        }

        let token = keys[0].replace("Bearer ", "");

        match validate_token(token.to_string()) {
            Ok(content) => {
                let repository = MongoRepository::<Admin>::new(
                    request
                    .guard::<Connection<MongoDb>>()
                    .await
                    .unwrap()
                    .default_database()
                    .unwrap()
                );

                let res = repository.find_by_id(content.sub).await;

                if let Some(admin) = res {
                    return Outcome::Success(admin);
                } else {
                    return Outcome::Forward(Status::Continue);
                }
            }
            Err(_) => {}
        }

        Outcome::Forward(Status::Continue)
    }
}
