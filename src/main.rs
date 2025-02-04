mod configurations;
mod controllers;
mod db;
mod model;
mod security;
mod unifi;
mod utils;

use configurations::config::ConfigApplication;
use controllers::admin_controller::{self, admin_page};
use controllers::client_controller::{self, client_connect_page, client_register};
use controllers::error_controller::handles;
use controllers::{approver_controller, config_controller};
use db::mongo_db::MongoDb;
use dotenv::dotenv;
use rocket::fs::FileServer;
use rocket::tokio::{
    self,
    sync::Mutex,
    time::{self, Duration},
};
use rocket::{Route, launch, routes};
use rocket_db_pools::Database;
use rocket_db_pools::mongodb::Client;
use std::env;
use std::sync::Arc;
use unifi::unifi::UnifiController;
use utils::monitoring::ClientsMonitoring;

///////////////////////////////////////////

#[launch]
async fn start() -> _ {
    // Starting environment variables
    dotenv().ok();


    // Creating an instance of the Configuration and Request Structure to the Unifi Controller
    let mut unifi = UnifiController::new(
        env::var("UNIFI_CONTROLLER_URL").expect("UNIFI_CONTROLLER_URL NOT DEFINED"),
        env::var("UNIFI_USER").expect("UNIFI_USER NOT DEFINED"),
        env::var("UNIFI_PASSWORD").expect("UNIFI_PASSWORD NOT DEFINED"),
    );

    // Trying to login to the Unifi Controller
    let _ = unifi.authentication_api().await;

    // Creating monitoring that will happen in X time to align with UniFi information
    let clone_unifi = unifi.clone();
    tokio::spawn(async move {
        let client =
            Client::with_uri_str(env::var("DATABASE_URL").expect("DATABASE_URL NOT DEFINED"))
                .await
                .unwrap();
        let db = client.default_database().unwrap();
        let mut monitoring = ClientsMonitoring::new(vec!["default".to_string()], db, clone_unifi);

        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            monitoring.all().await;
        }
    });

    // Rocket Server
    rocket::build()
        .attach(MongoDb::init())
        //
        .manage(Arc::new(Mutex::new(unifi)))
        //
        .register("/api", handles())
        //
        .mount(
            "/static",
            FileServer::from(env::var("STATIC_FILES_DIR").expect("STATIC_FILES_DIR NOT DEFINED")),
        )
        .mount("/", routes![client_register])
        .mount("/admin", routes![admin_page])
        .mount("/client", routes![client_connect_page])
        .mount("/api", api_routes())
}

pub fn api_routes() -> Vec<Route> {
    let mut routes = client_controller::routes();
    routes.append(&mut admin_controller::routes());
    routes.append(&mut approver_controller::routes());
    routes.append(&mut config_controller::routes());

    routes
}
