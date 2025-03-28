use crate::{
    configurations::config::ConfigApp, ldap::ldap::LdapConnection, model::{
        entity::{
            admin::Admin,
            approver::{Approver, ApproverCode, ApproverData, ApproverLogin, ApproverUpdate},
        },
        repository::{mongo_repository::MongoRepository, Repository},
    }, utils::{
        error::{BadRequest, CustomError, Error, Unauthorized},
        generator,
        responses::{Created, Ok, Response},
    }
};
use bcrypt::{DEFAULT_COST, hash, verify};
use bson::doc;
use rocket::{Route, State, delete, get, post, put, routes, serde::json::Json};

#[post("/approver", data = "<data>")]
pub async fn create_approver(
    data: Json<ApproverData>,
    repository: MongoRepository<Approver>,
    _admin: Admin,
    config: &State<ConfigApp>,
) -> Result<Created<()>, Unauthorized> {
    let config = config.read().await;

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
        secrete_code: approver.secrete_code,
    };
    approver.create_validity(config.approvers.validity_days_code.clone() as i64);

    let _ = repository.save(approver).await;

    Ok(Response::new_created(()))
}

#[get("/approver")]
pub async fn get_approvers(
    _admin: Admin,
    repository: MongoRepository<Approver>,
) -> Result<Ok<Vec<Approver>>, Unauthorized> {
    let mut entitys = repository.find_all().await;
    for i in 0..entitys.len() {
        let e = entitys.get_mut(i).unwrap();
        e.password = String::from("");
        e.secrete_code = String::from("");
    }

    Ok(Response::new_ok(entitys))
}

#[put("/approver/<id>", data = "<data>")]
pub async fn update_approver(
    data: Json<ApproverUpdate>,
    id: String,
    repository: MongoRepository<Approver>,
    _admin: Admin,
    config: &State<ConfigApp>,
) -> Result<Ok<()>, CustomError> {
    let config = config.read().await;
    let approver_data = data.into_inner();

    let op = repository.find_by_id(id).await;
    let mut approver;

    if let Some(a) = op {
        approver = a;
    } else {
        return Err(Error::new_bad_request("Approver Not Found"));
    }

    approver.email = approver_data.email.unwrap_or(approver.email);
    approver.password = approver_data
        .password
        .map(|p| hash(p, DEFAULT_COST).unwrap())
        .unwrap_or(approver.password);

    approver.secrete_code = approver_data
        .secrete_code
        .map(|c| {
            let new_code = hash(c, DEFAULT_COST).unwrap();
            approver.create_validity(config.approvers.validity_days_code.clone() as i64);
            new_code
        })
        .unwrap_or(approver.secrete_code);

    let _ = repository.update(approver).await;

    Ok(Response::new_ok(()))
}

#[put("/approver/code", data = "<data>")]
pub async fn generator_approver_code(
    data: Json<ApproverLogin>,
    repository: MongoRepository<Approver>,
    config: &State<ConfigApp>,
) -> Result<Ok<ApproverCode>, BadRequest> {
    let config = config.read().await;
    let code_size = config.approvers.code_size.clone();
    let just_numbers = config.approvers.just_numbers.clone();

    let op_approver = repository
        .find_one(doc! {
            "username" : data.username.clone()
        })
        .await;

    match op_approver {
        Some(mut approver) => {
            if approver.password.is_empty() {
                if let Some(v) = config.ldap.clone() {
                    let ldap = LdapConnection::new(v);
                    let auth = ldap.simple_authentication(&data.username, &data.password).await;
                    
                    if !auth {
                        return Err(Error::new_bad_request("Invalid username or password"));
                    }
                }
            }
            else {
                let ok = verify(data.password.clone(), approver.password.as_str()).unwrap_or(false);
                if !ok {
                    return Err(Error::new_bad_request("Invalid username or password"));
                }
            }
            

            let new_code = generator::generator_code(code_size, just_numbers);
            approver.secrete_code = hash(new_code.clone(), DEFAULT_COST).unwrap();
            approver.create_validity(config.approvers.validity_days_code.clone() as i64);

            let _ = repository.update(approver).await;
            Ok(Response::new_ok(ApproverCode::new(
                new_code,
                config.approvers.validity_days_code.clone(),
            )))
        }

        None => Err(Error::new_bad_request("Invalid username or password")),
    }
}

#[delete("/approver/<id>")]
pub async fn delete_approver(
    id: String,
    repository: MongoRepository<Approver>,
    _admin: Admin,
) -> Result<Ok<()>, Unauthorized> {
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
