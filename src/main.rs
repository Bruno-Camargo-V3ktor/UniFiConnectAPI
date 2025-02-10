mod configurations;
mod controllers;
mod db;
mod model;
mod security;
mod unifi;
mod utils;
mod ldap;

use configurations::config::ConfigApplication;
use controllers::admin_controller::{self, admin_page};
use controllers::client_controller::{self, client_connect_page, client_register};
use controllers::error_controller::handles;
use controllers::{approver_controller, config_controller, user_controller};
use db::mongo_db::MongoDb;
use rocket::fs::FileServer;
use rocket::tokio::{
    self,
    sync::Mutex,
    time::{self, Duration},
};
use rocket::{Route, launch, routes};
use rocket_db_pools::Database;
use rocket_db_pools::mongodb::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use unifi::unifi::UnifiController;
use utils::monitoring::ClientsMonitoring;

///////////////////////////////////////////

#[launch]
async fn start() -> _ {
    // Starting Configurations...
    let config = ConfigApplication::new();

    // Creating an instance of the Configuration and Request Structure to the Unifi Controller
    let unifi = UnifiController::new(
        config.unifi.url.clone(),
        config.unifi.username.clone(),
        config.unifi.password.clone(),
    )
    .await;

    // Starting monitoring clients
    tokio::spawn(monitoring_clients(unifi.clone(), config.clone()));

    // Rocket Server
    rocket::custom(config.to_rocket_config())
        .attach(MongoDb::init())
        //
        .manage(Arc::new(Mutex::new(unifi)))
        .manage(RwLock::new(config.clone()))
        //
        .register("/api", handles())
        //
        .mount("/static", FileServer::from(config.server.files_dir.clone()))
        .mount("/", routes![client_register])
        .mount("/admin", routes![admin_page])
        .mount("/client", routes![client_connect_page])
        .mount("/api", api_routes())
}

fn api_routes() -> Vec<Route> {
    let mut routes = client_controller::routes();
    routes.append(&mut admin_controller::routes());
    routes.append(&mut approver_controller::routes());
    routes.append(&mut user_controller::routes());
    routes.append(&mut config_controller::routes());

    routes
}

// Creating monitoring that will happen in X time to align with UniFi information
async fn monitoring_clients(unifi: UnifiController, config: ConfigApplication) {
    let client = Client::with_uri_str(config.database.get_formated_url())
        .await
        .unwrap();

    let db = client.default_database().unwrap();
    let mut monitoring = ClientsMonitoring::new(db, unifi);

    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        monitoring.all().await;
    }
}
