use rocket::{ launch, routes };

mod unifi;
use unifi::unifi_controller::UnifiController;

use std::env;
use std::sync::{Arc, Mutex};
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
        .mount( "/", routes![] )
        .mount( "/api", routes![] )
        .manage( Arc::new( Mutex::new(unifi) ) )

}
