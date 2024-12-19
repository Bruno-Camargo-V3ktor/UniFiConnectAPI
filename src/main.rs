mod unifi;
use unifi::unifi_controller::UnifiController;

use rocket::{ launch, routes, get, post };
use dotenv::dotenv;
use std::env;
use std::sync::{Arc, Mutex};



#[ launch ]
async fn start() -> _ {
    dotenv().ok(); // Starting environment variables

    // Creating an instance of the Configuration and Request Structure to the Unifi Controller
    let mut unifi = UnifiController::new(
        env::var("UNIFI_CONTROLLER_URL").unwrap(),
        env::var("UNIFI_USER").unwrap(),
        env::var("UNIFI_PASSWORD").unwrap()
    );

    let _ = unifi.authentication_api().await.unwrap();

    rocket::build()
        .mount("/", routes![])
        .mount("/api", routes![])
        .manage( Arc::new( Mutex::new(unifi) ) )

}
