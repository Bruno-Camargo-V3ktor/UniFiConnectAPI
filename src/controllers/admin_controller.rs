use bcrypt::{DEFAULT_COST, hash, verify};
use bson::doc;
use rocket::fairing::Result;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::{Route, State, delete, get, post, put, routes};

use crate::configurations::config::ConfigApp;
use crate::ldap::ldap::LdapConnection;
use crate::model::entity::admin::{Admin, AdminData, AdminLogin};
use crate::model::repository::Repository;
use crate::model::repository::mongo_repository::MongoRepository;
use crate::security::auth_jwt::create_token;
use crate::utils::error::{BadRequest, CustomError, Error, Unauthorized};
use crate::utils::responses::{Accepted, Created, Ok, Response};

// ENDPOINTS
#[get("/<_..>")]
pub async fn admin_page(config: &State<ConfigApp>) -> Result<NamedFile, ()> {
    let config = config.read().await;
    let mut path = config.server.files_dir.clone();
    path.push_str("/admin/index.html");

    Ok(NamedFile::open(path).await.expect("Admin Page Not Found"))
}

#[post("/admin/login", data = "<data>")]
pub async fn admin_login(
    data: Json<AdminLogin>,
    repository: MongoRepository<Admin>,
    config: &State<ConfigApp>,
) -> Result<Accepted<String>, BadRequest> {
    let config = config.read().await;

    let res = repository
        .find_one(doc! {
           "username" : &data.username
        })
        .await;

    match res {
        Some(admin) => {

            match admin.password {
                Some(p) => {
                    let check = verify(&data.password, &p);
                    if let Ok(b) = check {
                        if b {
                            return Ok(Response::new_accepted(create_token(
                                &admin.id,
                                config.server.secret_key.clone(),
                                config.admins.token_expirantion.clone() as u64,
                            )));
                         }
                    }
                }

                None => {
                    if let Some(v) = config.ldap.clone() {
                        let ldap = LdapConnection::new(v);
                        let auth = ldap.simple_authentication(&data.username, &data.password).await;

                        if auth {
                            return Ok(Response::new_accepted(create_token(
                                &admin.id,
                                config.server.secret_key.clone(),
                                config.admins.token_expirantion.clone() as u64,
                            )));
                        }
                    }
                }
            }

        }

        None => {}
    }

    Err(Error::new_bad_request("Invalid Username or Password"))
}

#[post("/admin", data = "<data>")]
pub async fn create_admin(
    data: Json<AdminData>,
    repository: MongoRepository<Admin>,
    _admin: Admin,
) -> Result<Created<()>, CustomError> {
    let data = data.into_inner();

    let res = repository
        .find_one(doc! { "username": data.username.clone() })
        .await;
    match res {
        Some(_) => return Err(Error::new_bad_request("Username already registered")),
        None => {}
    }

    let mut new_admin = Admin {
        id: "".to_string(),
        name: data.name,
        username: data.username,
        password: data.password,
    };

    new_admin.password = match new_admin.password {
        Some(p) => Some(hash(p.as_str(), DEFAULT_COST).unwrap()),
        None => {
            return Err(Error::new_bad_request("field 'password' not found"));
        }
    };

    let _ = repository.save(new_admin).await;

    Ok(Response::new_created(()))
}

#[put("/admin", data = "<data>")]
pub async fn update_admin(
    data: Json<Admin>,
    repository: MongoRepository<Admin>,
    mut admin: Admin,
) -> Result<Ok<()>, CustomError> {
    let admin_data = data.into_inner();

    if !admin_data.id.is_empty() {
        let res = repository.find_by_id(admin_data.id).await;
        if let Some(a) = res {
            admin = a;
        } else {
            return Err(Error::new_not_found("Admin User Not Found"));
        }
    }

    admin.name = admin_data.name;

    if admin.username != admin_data.username {
        admin.username = {
            let res = repository
                .find_one(doc! { "username": admin_data.username.clone() })
                .await;
            match res {
                Some(_) => return Err(Error::new_bad_request("Username already registered")),
                None => admin_data.username,
            }
        };
    }

    admin.password = match admin.password {
        Some(p) => Some(hash(p.as_str(), DEFAULT_COST).unwrap()),
        None => admin.password.clone(),
    };

    let _ = repository.update(admin).await;
    Ok(Response::new_ok(()))
}

#[delete("/admin/<id>")]
pub async fn delete_admin(
    id: String,
    repository: MongoRepository<Admin>,
    _admin: Admin,
) -> Result<Ok<()>, Unauthorized> {
    let _ = repository.delete_by_id(id).await;

    Ok(Response::new_ok(()))
}

#[get("/admin")]
pub async fn get_admins(
    repository: MongoRepository<Admin>,
    _admin: Admin,
) -> Result<Ok<Vec<Admin>>, Unauthorized> {
    let mut entites = repository.find_all().await;
    for i in 0..entites.len() {
        let e = entites.get_mut(i).unwrap();
        e.password = None;
    }

    Ok(Response::new_ok(entites))
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![admin_login, create_admin, update_admin, delete_admin, get_admins]
}
