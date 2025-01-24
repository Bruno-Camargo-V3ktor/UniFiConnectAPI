mod controllers;
mod db;
mod model;
mod security;
mod unifi;
mod utils;

use controllers::admin_controller::{create_admin, delete_admin, login, update_admin};
use controllers::approver_controller::{create_approver, delete_approver, update_approver};
use controllers::guest_controller::{get_guests, guest_connection_request, guest_register};
use db::mongo_db::MongoDb;
use rocket::fs::FileServer;
use rocket::{launch, routes};
use rocket_db_pools::Database;
use rocket_db_pools::mongodb::Client;
use unifi::unifi::UnifiController;

use rocket::tokio::{
    self,
    sync::Mutex,
    time::{self, Duration},
};

use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use utils::guest_utils::GuestMonitoring;

///////////////////////////////////////////

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

    // Creating monitoring that will happen in X time to align with UniFi information
    let clone_unifi = unifi.clone();
    tokio::spawn(async move {
        let client = Client::with_uri_str(env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        let db = client.default_database().unwrap();
        let mut monitoring = GuestMonitoring::new(vec!["default".to_string()], db, clone_unifi);

        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            println!("Teste");
            monitoring.all().await;
        }
    });

    rocket::build()
        .attach(MongoDb::init())
        .manage(Arc::new(Mutex::new(unifi)))
        .mount(
            "/guest",
            FileServer::from(env::var("GUEST_LOGIN_PAGE").unwrap()),
        )
        .mount(
            "/admin",
            FileServer::from(env::var("ADMIN_LOGIN_PAGE").unwrap()),
        )
        .mount("/guest", routes![guest_register])
        .mount("/api", routes![
            guest_connection_request,
            get_guests,
            login,
            create_admin,
            update_admin,
            delete_admin,
            create_approver,
            update_approver,
            delete_approver
        ])
}
