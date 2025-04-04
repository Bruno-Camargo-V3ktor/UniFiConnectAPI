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
use ldap::ldap::LdapConnection;
use rocket::fs::FileServer;
use rocket::tokio::{
    self,
    sync::Mutex,
    time::{self, Duration},
};
use rocket::{Route, launch, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use rocket_db_pools::mongodb::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use unifi::unifi::UnifiController;
use utils::monitoring::{ClientsMonitoring, LdapMonitoring};

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

    // Starting scan LDAP
    tokio::spawn(monitoring_ldap(config.clone()));

    // Starting monitoring clients
    tokio::spawn(monitoring_clients(unifi.clone(), config.clone()));

    // CORS Configuration
    let allowed_origins = AllowedOrigins::all();

    let cors = CorsOptions {
        allowed_origins,
        // Permite os métodos que você deseja (incluindo PUT, OPTIONS, etc.)
        allowed_methods: ["PUT", "POST", "GET", "OPTIONS"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect(),
        // Permite os cabeçalhos necessários (ex: Content-Type)
        allowed_headers: AllowedHeaders::some(&["Authorization", "Content-Type", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors().unwrap();

    // Rocket Server
    rocket::custom(config.to_rocket_config())
        .attach(cors)
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

// Creating monitoring that will take place in X amount of time to integrate with LDAP
async fn monitoring_ldap(config: ConfigApplication) {
    if let Some(ldap) = config.ldap.clone() {

        let client = Client::with_uri_str(config.database.get_formated_url())
        .await
        .unwrap();

        let db = client.default_database().unwrap();

        let connection = LdapConnection::new(ldap.clone());
        let monitoring = LdapMonitoring::new(db, ldap.clone());
        
        let mut interval = time::interval(Duration::from_secs(20));
        let conn_res = connection.create_connection().await;

        match conn_res {
            Ok(mut conn) => {
                loop {
                    monitoring.scan_admins(&mut conn, &connection).await;
                    monitoring.scan_approvers(&mut conn, &connection, &config.approvers).await;
                    monitoring.scan_users(&mut conn, &connection, &config.users).await; 
                    interval.tick().await;
                }
            }

            Err(e) => println!("{e:?}")
        }

        
    }
}


// Creating monitoring that will happen in X time to align with UniFi information
async fn monitoring_clients(unifi: UnifiController, config: ConfigApplication) {
    let client = Client::with_uri_str(config.database.get_formated_url())
        .await
        .unwrap();

    let db = client.default_database().unwrap();
    let mut monitoring = ClientsMonitoring::new(db, unifi, config.clients.clone());

    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        monitoring.all().await;
    }
}
