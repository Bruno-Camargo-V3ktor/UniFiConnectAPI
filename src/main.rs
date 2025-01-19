mod controllers;
mod db;
mod model;
mod security;
mod unifi;
mod utils;

use controllers::admin_controller::{admin_page, create_admin, delete_admin, login, update_admin};
use controllers::guest_controller::{guest_connection_request, guest_page, guest_register};
use db::mongo_db::MongoDb;
use rocket::{launch, routes};
use rocket_db_pools::Database;
use unifi::unifi::UnifiController;

use tokio::sync::Mutex;

use dotenv::dotenv;
use std::env;
use std::sync::Arc;

#[launch]
async fn start() -> _ {
    // Starting environment variables
    dotenv().ok();

    // Creating an instance of the Configuration and Request Structure to the Unifi Controller
    let mut unifi = UnifiController::new(
        env::var("UNIFI_CONTROLLER_URL").unwrap(),
        env::var("UNIFI_USER").unwrap(),
        env::var("UNIFI_PASSWORD").unwrap(),
    );

    // Trying to login to the Unifi Controller
    let _ = unifi.authentication_api().await;

    rocket::build()
        .attach(MongoDb::init())
        .manage(Arc::new(Mutex::new(unifi)))
        .mount("/admin", routes![admin_page])
        .mount("/guest", routes![guest_page, guest_register])
        .mount("/api", routes![
            guest_connection_request,
            login,
            create_admin,
            update_admin,
            delete_admin,
        ])
}
