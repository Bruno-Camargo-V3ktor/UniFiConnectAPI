use crate::{
    model::{
        entity::{admin::Admin, approver::{Approver, ApproverData}},
        repository::{approver_repository::ApproverRepository, Repository},
    },
    utils::error::Error,
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Local;
use rocket::http::Status;
use rocket::{delete, post, put, response::status::Custom, serde::json::Json};

#[post("/approver", data = "<data>")]
pub async fn create_approver(
    data: Json<ApproverData>,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    if admin.is_none() {
        return Err(Error::new_with_custom(
            "Unauthorized user",
            Local::now().to_string(),
            401,
        ));
    }

    let mut approver = data.into_inner();
    approver.secrete_code = hash(approver.secrete_code.as_str(), DEFAULT_COST).unwrap();

    let approver = Approver { 
        id: String::new(), 
        username: approver.username, 
        email:  approver.email, 
        secrete_code:  approver.secrete_code 
    };

    let _ = repository.save(approver).await;

    Ok(Custom(Status::Created, ()))
}

#[put("/approver", data = "<data>")]
pub async fn update_approver(
    data: Json<Approver>,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    if admin.is_none() {
        return Err(Error::new_with_custom(
            "Unauthorized user",
            Local::now().to_string(),
            401,
        ));
    }

    let mut approver = data.into_inner();
    approver.secrete_code = hash(approver.secrete_code.as_str(), DEFAULT_COST).unwrap();
    let _ = repository.update(approver).await;

    Ok(Custom(Status::Ok, ()))
}

#[delete("/approver/<id>")]
pub async fn delete_approver(
    id: String,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Custom<()>, Custom<Json<Error>>> {
    if admin.is_none() {
        return Err(Error::new_with_custom(
            "Unauthorized user",
            Local::now().to_string(),
            401,
        ));
    }

    let _ = repository.delete_by_id(id).await;

    Ok(Custom(Status::Ok, ()))
}
