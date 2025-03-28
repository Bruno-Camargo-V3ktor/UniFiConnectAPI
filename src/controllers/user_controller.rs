use crate::{
    configurations::config::ConfigApp,
    ldap::ldap::LdapConnection,
    model::{
        entity::{
            admin::Admin,
            client::{Client, ClientStatus},
            user::{User, UserLogin, UserUpdate},
        },
        repository::{mongo_repository::MongoRepository, Repository},
    },
    unifi::unifi::UnifiController,
    utils::{
        error::{BadRequest, Error, NotFound, Unauthorized},
        responses::{Accepted, Created, Ok, Response},
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use bson::doc;
use rocket::{Route, State, delete, get, http::CookieJar, post, put, routes, serde::json::Json};

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

    if !config.users.registrations_open && admin.is_none() {
        match admin {
            Some(_) => {}
            None => return Err(Error::new_bad_request("Closed")),
        }
    }

    if user.username.len() < 3 && user.email.len() < 10 && user.password.len() < 6 {
        return Err(Error::new_bad_request("Invalid field(s)"));
    }
    if repo
        .find_one(doc! { "username": user.username.clone() })
        .await.is_some()
    {
        return Err(Error::new_bad_request("Username is already in use"));
    }

    user.password = hash(user.password, DEFAULT_COST).unwrap();

    let _ = repo.save(user).await;

    Ok(Response::new_created(String::from("User Created")))
}

#[post("/user/login", data = "<data>")]
pub async fn login_user(
    data: Json<UserLogin>,
    cookies: &CookieJar<'_>,
    mut unifi: UnifiController,
    user_repo: MongoRepository<User>,
    client_repo: MongoRepository<Client>,
    config: &State<ConfigApp>,
) -> Result<Accepted<String>, BadRequest> {
    let config = config.read().await;

    match user_repo
        .find_one(doc! {"username": data.username.clone()})
        .await
    {
        Some(user) => {
            if user.password.is_empty() {
                if let Some(v) = config.ldap.clone() {
                    let ldap = LdapConnection::new(v);
                    let auth = ldap
                        .simple_authentication(&data.username, &data.password)
                        .await;

                    if !auth {
                        return Err(Error::new_bad_request("Invalid username or password"));
                    }
                }
            } else {
                let ok = verify(&data.password, &user.password).unwrap_or(false);
                if !ok {
                    return Err(Error::new_bad_request("Invalid username or password"));
                }
            }

            let mut new_client = Client::new_with_data(&user.data);

            let mac = cookies.get("id").unwrap().value().to_string();
            let site = cookies.get("site").unwrap().value().to_string();
            let minutes: u16 = config.clients.time_connection as u16;

            new_client.site = site.clone();
            new_client.mac = mac.clone();
            new_client.time_connection = minutes.to_string();
            new_client.status = ClientStatus::Approved;

            unifi.conect_client(&new_client).await;
            let _ = client_repo.save(new_client).await;

            Ok(Response::new_accepted(String::from("Connection Approved")))
        }

        None => {
            Err(Error::new_bad_request("Username or password invalid"))
        }
    }
}

#[get("/user")]
pub async fn get_users(
    _admin: Admin,
    repo: MongoRepository<User>,
) -> Result<Ok<Vec<User>>, Unauthorized> {
    let mut users = repo.find_all().await;
    users.iter_mut().for_each(|u| u.password = String::new());

    Ok(Response::new_ok(users))
}

#[put("/user/<id>", data = "<data>")]
pub async fn update_user(
    _admin: Admin,
    id: String,
    repo: MongoRepository<User>,
    data: Json<UserUpdate>,
) -> Result<Ok<()>, NotFound> {
    let data = data.into_inner();
    let user = repo.find_by_id(id).await;

    if let Some(mut u) = user {
        if u.password.is_empty() {
            u.data = data.data.unwrap_or(u.data.clone());
            repo.update(u).await;
            return Ok(Response::new_ok(()));
        }

        u.password = data
            .password
            .map(|c| hash(c, DEFAULT_COST).unwrap())
            .unwrap_or(u.password);
        u.email = data.email.unwrap_or(u.email);
        u.data = data.data.unwrap_or(u.data);

        repo.update(u).await;
        return Ok(Response::new_ok(()));
    }

    Err(Error::new_not_found("User not found"))
}

#[delete("/user/<id>")]
pub async fn delete_user(
    _admin: Admin,
    id: String,
    repo: MongoRepository<User>,
) -> Result<Ok<()>, NotFound> {
    let res = repo.delete_by_id(id).await;

    if res {
        Ok(Response::new_ok(()))
    } else {
        Err(Error::new_not_found("User not Found"))
    }
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![create_user, login_user, get_users, update_user, delete_user]
}
