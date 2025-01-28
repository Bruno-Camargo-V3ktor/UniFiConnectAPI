use crate::{
    model::{
        entity::{
            admin::Admin,
            approver::{Approver, ApproverData, ApproverUpdate},
        },
        repository::{Repository, approver_repository::ApproverRepository},
    },
    utils::{
        error::{CustomError, Error, Unauthorized},
        responses::{Created, Ok, Response},
    },
};
use bcrypt::{DEFAULT_COST, hash};
use rocket::{Route, delete, get, post, put, routes, serde::json::Json};

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
    approver.password = hash(approver.password.as_str(), DEFAULT_COST).unwrap();

    let mut approver = Approver {
        id: String::new(),
        username: approver.username,
        email: approver.email,
        password: approver.password,
        validity: None,
        secrete_code: approver.secrete_code,
    };
    approver.create_validity();

    let _ = repository.save(approver).await;

    Ok(Response::new_created(()))
}

#[get("/approver")]
pub async fn get_approvers(
    admin: Option<Admin>,
    repository: ApproverRepository,
) -> Result<Ok<Vec<Approver>>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let mut entitys = repository.find_all().await;
    for i in 0..entitys.len() {
        let e = entitys.get_mut(i).unwrap();
        e.password = String::from("");
        e.secrete_code = String::from("");
    }

    Ok(Response::new_ok(entitys))
}

#[put("/approver", data = "<data>")]
pub async fn update_approver(
    data: Json<ApproverUpdate>,
    repository: ApproverRepository,
    admin: Option<Admin>,
) -> Result<Ok<()>, CustomError> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let approver_data = data.into_inner();

    let op = repository.find_by_id(approver_data.id).await;
    let mut approver;

    if let Some(a) = op {
        approver = a;
    } else {
        return Err(Error::new_bad_request("Approver Not Found"));
    }

    approver.email = approver_data.email.unwrap_or(approver.email.clone());
    approver.username = approver_data.username.unwrap_or(approver.username.clone());
    approver.password = {
        if let Some(p) = approver_data.password {
            hash(p.as_str(), DEFAULT_COST).unwrap();
        }
        approver.password.clone()
    };
    approver.secrete_code = {
        if let Some(s) = approver_data.secrete_code {
            hash(s.as_str(), DEFAULT_COST).unwrap();
            approver.create_validity();
        }
        approver.secrete_code.clone()
    };

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

// Functions
pub fn routes() -> Vec<Route> {
    routes![
        create_approver,
        update_approver,
        delete_approver,
        get_approvers
    ]
}
