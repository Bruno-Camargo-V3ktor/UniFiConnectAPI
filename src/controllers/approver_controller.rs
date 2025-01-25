use crate::{
    model::{
        entity::{
            admin::Admin,
            approver::{Approver, ApproverData},
        },
        repository::{Repository, approver_repository::ApproverRepository},
    },
    utils::{
        error::{Error, Unauthorized},
        responses::{Created, Ok, Response},
    },
};
use bcrypt::{DEFAULT_COST, hash};
use rocket::{delete, post, put, serde::json::Json};

#[post("/approver", data = "<data>")]
pub async fn create_approver(
    data: Json<ApproverData>,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Created<()>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let mut approver = data.into_inner();
    approver.secrete_code = hash(approver.secrete_code.as_str(), DEFAULT_COST).unwrap();

    let approver = Approver {
        id: String::new(),
        username: approver.username,
        email: approver.email,
        secrete_code: approver.secrete_code,
    };

    let _ = repository.save(approver).await;

    Ok(Response::new_created(()))
}

#[put("/approver", data = "<data>")]
pub async fn update_approver(
    data: Json<Approver>,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Ok<()>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let mut approver = data.into_inner();
    approver.secrete_code = hash(approver.secrete_code.as_str(), DEFAULT_COST).unwrap();
    let _ = repository.update(approver).await;

    Ok(Response::new_ok(()))
}

#[delete("/approver/<id>")]
pub async fn delete_approver(
    id: String,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Ok<()>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let _ = repository.delete_by_id(id).await;

    Ok(Response::new_ok(()))
}
