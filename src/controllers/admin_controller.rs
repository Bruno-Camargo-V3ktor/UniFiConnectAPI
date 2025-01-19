use bcrypt::{DEFAULT_COST, hash};
use chrono::Local;
use rocket::fairing::Result;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status::{Accepted, Custom, NotFound};
use rocket::serde::json::Json;
use rocket::{delete, get, post, put};
use rocket_db_pools::Connection;
use std::env;

use crate::db::mongo_db::MongoDb;
use crate::model::entity::admin::{Admin, AdminData, AdminLogin};
use crate::model::repository::Repository;
use crate::model::repository::admin_repositoy::AdminRepository;
use crate::utils::error::Error;

// ENDPOINTS
#[get("/<_..>", format = "text/html")]
pub async fn admin_page() -> Result<NamedFile, NotFound<String>> {
    let index_path = env::var("ADMIN_LOGIN_PAGE").unwrap();

    NamedFile::open(index_path)
        .await
        .map_err(|_| NotFound("Page not found".to_string()))
}

#[post("/admin/login", data = "<data>")]
pub async fn login(data: Json<AdminLogin>) -> Result<Accepted<String>, Custom<Json<Error>>> {
    Ok(Accepted(String::from("token...")))
}

#[post("/admin", data = "<data>")]
pub async fn create_admin(
    data: Json<AdminData>,
    db: Connection<MongoDb>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    let repository = AdminRepository {
        database: db.default_database().unwrap(),
        name: "Admins".to_string(),
    };

    let data = data.into_inner();
    let mut new_admin = Admin {
        id: "".to_string(),
        name: data.name,
        username: data.username,
        password: data.password,
    };

    new_admin.password = match new_admin.password {
        Some(p) => Some(hash(p.as_str(), DEFAULT_COST).unwrap()),
        None => {
            return Err(Custom(
                Status::BadRequest,
                Json(Error {
                    time: Local::now().naive_local().to_string(),
                    err: String::from("field 'password' not found"),
                    status: 404,
                }),
            ));
        }
    };

    let _ = repository.save(new_admin).await;

    Ok(Custom(Status::Created, ()))
}

#[put("/admin", data = "<data>")]
pub async fn update_admin(
    data: Json<Admin>,
    db: Connection<MongoDb>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    let repository = AdminRepository {
        database: db.default_database().unwrap(),
        name: "Admins".to_string(),
    };

    let mut admin = data.into_inner();

    admin.password = match admin.password {
        Some(p) => Some(hash(p.as_str(), DEFAULT_COST).unwrap()),
        None => {
            return Err(Custom(
                Status::BadRequest,
                Json(Error {
                    time: Local::now().naive_local().to_string(),
                    err: String::from("field 'password' not found"),
                    status: 404,
                }),
            ));
        }
    };

    let _ = repository.update(admin).await;
    Ok(Custom(Status::Ok, ()))
}

#[delete("/admin/<id>")]
pub async fn delete_admin(
    id: String,
    db: Connection<MongoDb>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    let repository = AdminRepository {
        database: db.default_database().unwrap(),
        name: "Admins".to_string(),
    };

    let _ = repository.delete_by_id(id).await;

    Ok(Custom(Status::Ok, ()))
}
