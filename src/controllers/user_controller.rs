use crate::{
    configurations::config::ConfigApp,
    model::{
        entity::{admin::Admin, user::User},
        repository::{Repository, mongo_repository::MongoRepository},
    },
    utils::{
        error::{BadRequest, Error},
        responses::{Created, Response},
    },
};
use bcrypt::{DEFAULT_COST, hash};
use bson::doc;
use rocket::{Route, State, post, routes, serde::json::Json};

// Endpoints
#[post("/user", data = "<data>")]
pub async fn create_user(
    admin: Option<Admin>,
    repo: MongoRepository<User>,
    config: &State<ConfigApp>,
    data: Json<User>,
) -> Result<Created<String>, BadRequest> {
    let config = config.read().await;
    let mut user = data.into_inner();

    if !config.users.registrations_open {
        match admin {
            Some(_) => {}
            None => return Err(Error::new_bad_request("Closed")),
        }
    }

    if user.username.len() < 3 && user.email.len() < 10 && user.password.len() < 6 {
        return Err(Error::new_bad_request("Invalid field(s)"));
    }
    if let Some(_) = repo
        .find_one(doc! { "username": user.username.clone() })
        .await
    {
        return Err(Error::new_bad_request("Username is already in use"));
    }

    if let None = admin {
        user.data.client_type = config.users.default_group.clone();
    }
    user.password = hash(user.password, DEFAULT_COST).unwrap();

    let _ = repo.save(user).await;

    Ok(Response::new_created(String::from("User Created")))
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![create_user]
}
