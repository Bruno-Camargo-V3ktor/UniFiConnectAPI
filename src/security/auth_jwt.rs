use crate::{
    configurations::config::ConfigApp,
    db::mongo_db::MongoDb,
    model::{
        entity::admin::Admin,
        repository::{mongo_repository::MongoRepository, Repository},
    },
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result};
use rocket::{
    http::Status, request::{FromRequest, Outcome, Request}, serde::{Deserialize, Serialize}, State
};
use rocket_db_pools::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

// Functions
pub fn create_token(user_id: &str, key: String, hours: u64) -> String {
    let minutes = hours * 60;

    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(60 * minutes);

    let content = Claims {
        sub: user_id.to_string(),
        exp: expiration.as_secs() as usize,
    };

    encode(
        &Header::default(),
        &content,
        &EncodingKey::from_secret(key.as_bytes()),
    )
    .unwrap()
}

pub fn validate_token(token: String, key: String) -> Result<Claims> {
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
        let config = request
            .guard::<&State<ConfigApp>>()
            .await
            .unwrap()
            .read()
            .await;

        if keys.is_empty() {
            return Outcome::Error((Status::BadRequest, ()));
        }

        let token = keys[0].replace("Bearer ", "");
        
        if let Ok(content) = validate_token(token.to_string(), config.server.secret_key.clone()) {
            let repository = MongoRepository::<Admin>::new( 
                request
                .guard::<Connection<MongoDb>>()
                .await
                .unwrap()
                .default_database()
                .unwrap()
            );

            let res = repository.find_by_id(content.sub).await;
            if let Some(admin) = res { return Outcome::Success(admin); }
        }

        return Outcome::Error((Status::Unauthorized, ()));
    }
}
