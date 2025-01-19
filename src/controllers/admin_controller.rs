use rocket::fairing::Result;
use rocket::fs::NamedFile;
use rocket::response::status::{Accepted, Custom, NotFound};
use rocket::serde::json::Json;
use rocket::{get, post};
use std::env;

use crate::model::entity::admin::AdminLogin;
use crate::utils::error::Error;

// ENDPOINTS
#[get("/<_..>", format = "text/html")]
pub async fn admin_page() -> Result<NamedFile, NotFound<String>> {
    let index_path = env::var("GUEST_LOGIN_PAGE").unwrap();

    NamedFile::open(index_path)
        .await
        .map_err(|_| NotFound("Page not found".to_string()))
}

#[post("/admin/login", data = "<data>")]
pub async fn login(data: Json<AdminLogin>) -> Result<Accepted<String>, Custom<Json<Error>>> {
    Ok(Accepted(String::from("token...")))
}
