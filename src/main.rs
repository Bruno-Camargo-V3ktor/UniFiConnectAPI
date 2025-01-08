mod unifi;
mod utils;
mod controllers;
mod model;

use rocket::{ launch, routes };

use unifi::unifi::UnifiController;
use controllers::guest_controller::{guest_register, guest_page, guest_connection_request};

use tokio::sync::Mutex;

use std::env;
use std::sync::Arc;
use dotenv::dotenv;


#[ launch ]
async fn start() -> _ {
    // Starting environment variables
    dotenv().ok();

    // Creating an instance of the Configuration and Request Structure to the Unifi Controller
    let mut unifi = UnifiController::new(
        env::var("UNIFI_CONTROLLER_URL").unwrap(),
        env::var("UNIFI_USER").unwrap(),
        env::var("UNIFI_PASSWORD").unwrap()
    );

    // Trying to login to the Unifi Controller
    let _ = unifi.authentication_api().await.unwrap();

    rocket::build()
        .mount( "/guest", routes![guest_page, guest_register] )
        .mount( "/api", routes![guest_connection_request] )
        .manage( Arc::new( Mutex::new(unifi) ) )

}
