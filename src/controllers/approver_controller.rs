use crate::{
    model::{
        entity::{
            admin::Admin,
            approver::{Approver, ApproverCode, ApproverData, ApproverLogin, ApproverUpdate},
        },
        repository::{Repository, mongo_repository::MongoRepository},
    },
    utils::{
        error::{BadRequest, CustomError, Error, Unauthorized},
        generator,
        responses::{Created, Ok, Response},
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use bson::doc;
use rocket::{Route, delete, get, post, put, routes, serde::json::Json};
use std::env;

#[post("/approver", data = "<data>")]
pub async fn create_approver(
    data: Json<ApproverData>,
    repository: MongoRepository<Approver>,
    admin: Option<Admin>,
) -> Result<Created<()>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let mut approver = data.into_inner();

    let res = repository
        .find_one(doc! { "username": approver.username.clone() })
        .await;
    match res {
        Some(_) => return Err(Error::new_bad_request("Username already registered")),
        None => {}
    }

    approver.secrete_code = hash(approver.secrete_code.as_str(), DEFAULT_COST).unwrap();
    approver.password = hash(approver.password.as_str(), DEFAULT_COST).unwrap();

    let mut approver = Approver {
        id: String::new(),
        username: approver.username,
        email: approver.email,
        password: approver.password,
        validity: None,
        approved_types: vec!["Guest".to_string()],
        secrete_code: approver.secrete_code,
    };
    approver.create_validity();

    let _ = repository.save(approver).await;

    Ok(Response::new_created(()))
}

#[get("/approver")]
pub async fn get_approvers(
    admin: Option<Admin>,
    repository: MongoRepository<Approver>,
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
    repository: MongoRepository<Approver>,
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

    if let Some(u) = approver_data.username {
        if approver.username != u {
            let res = repository.find_one(doc! { "username": u.clone() }).await;
            match res {
                Some(_) => return Err(Error::new_bad_request("Username already registered")),
                None => {
                    approver.username = u;
                }
            }
        }
    }

    approver.password = {
        let mut new_password = approver.password.clone();
        if let Some(p) = approver_data.password {
            new_password = hash(p.as_str(), DEFAULT_COST).unwrap();
        }
        new_password
    };

    approver.secrete_code = {
        let mut new_code = approver.secrete_code.clone();
        if let Some(s) = approver_data.secrete_code {
            new_code = hash(s.as_str(), DEFAULT_COST).unwrap();
            approver.create_validity();
        }
        new_code
    };

    approver.approved_types = {
        if let Some(s) = approver_data.approved_types {
            approver.approved_types = s;
        }
        approver.approved_types
    };

    let _ = repository.update(approver).await;

    Ok(Response::new_ok(()))
}

#[put("/approver/code", data = "<data>")]
pub async fn generator_approver_code(
    admin: Option<Admin>,
    data: Json<ApproverLogin>,
    repository: MongoRepository<Approver>,
) -> Result<Ok<ApproverCode>, BadRequest> {
    let code_size = env::var("APPROVAL_CODE_SIZE")
        .unwrap_or("8".to_string())
        .parse::<u8>()
        .expect("APPROVAL_CODE_SIZE NOT NUMBER");
    let op_approver = repository
        .find_one(doc! {
            "username" : data.username.clone()
        })
        .await;

    match op_approver {
        Some(mut approver) => {
            if let None = admin {
                let ok = verify(data.password.clone(), approver.password.as_str()).unwrap_or(false);
                if !ok {
                    return Err(Error::new_bad_request("Invalid username or password"));
                }
            }

            let new_code = generator::generator_code(code_size);
            approver.secrete_code = hash(new_code.clone(), DEFAULT_COST).unwrap();
            approver.create_validity();

            let _ = repository.update(approver).await;
            Ok(Response::new_ok(ApproverCode::new(new_code)))
        }

        None => Err(Error::new_bad_request("Invalid username or password")),
    }
}

#[delete("/approver/<id>")]
pub async fn delete_approver(
    id: String,
    repository: MongoRepository<Approver>,
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
        get_approvers,
        generator_approver_code,
    ]
}
