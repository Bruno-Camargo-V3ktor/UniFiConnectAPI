use std::process;

use crate::{
    configurations::config::{ConfigApp, ConfigApplication, ConfigUpdate},
    model::entity::admin::Admin,
    utils::{
        error::Unauthorized,
        responses::{Ok, Response},
    },
};
use rocket::tokio::{
    self,
    time::{Duration, sleep},
};
use rocket::{Route, State, get, put, routes, serde::json::Json};

// ENDPOINTS
#[get("/config")]
pub async fn get_configs(
    _admin: Admin,
    config: &State<ConfigApp>,
) -> Result<Ok<ConfigApplication>, Unauthorized> {
    let config = config.read().await;
    Ok(Response::new_ok(config.to_owned()))
}

#[put("/config", data = "<data>")]
pub async fn update_configs(
    _admin: Admin,
    data: Json<ConfigUpdate>,
    config: &State<ConfigApp>,
) -> Result<Ok<()>, Unauthorized> {
    let data = data.into_inner();
    let mut config = config.write().await;

    config.admins = data.admins.unwrap_or(config.admins.clone());
    config.approvers = data.approvers.unwrap_or(config.approvers.clone());
    config.clients = data.clients.unwrap_or(config.clients.clone());
    config.users = data.users.unwrap_or(config.users.clone());
    config.unifi = data.unifi.clone().unwrap_or(config.unifi.clone());
    config.database = data.database.clone().unwrap_or(config.database.clone());
    config.server = data.server.clone().unwrap_or(config.server.clone());
    config.ldap = data.ldap.clone().unwrap_or(config.ldap.clone());

    if data.server.is_some() || data.unifi.is_some() || data.database.is_some() {
        tokio::spawn(async {
            sleep(Duration::from_secs(5)).await;
            process::exit(0);
        });
    }

    config.save();
    Ok(Response::new_ok(()))
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![get_configs, update_configs]
}
