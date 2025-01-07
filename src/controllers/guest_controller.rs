use rocket::{ State, get, post };
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::response::{ Redirect, status::{BadRequest, NotFound, Custom, Accepted} };
use rocket::http::CookieJar;
use rocket::fs::NamedFile;

use tokio::sync::Mutex;

use crate::UnifiController;
use crate::utils::error::Error;

use std::env;
use std::sync::Arc;

#[ derive(Deserialize, Serialize) ]
pub struct SecretCode {
    code: u16
}

// ENDPOINTS
#[ get( "/index/<_..>" ) ]
pub async fn guest_page( cookies: &CookieJar<'_> ) -> Result< NamedFile, NotFound<String> > {
    let index_path = env::var( "GUEST_LOGIN_PAGE" ).unwrap();

    println!( "{:?}", cookies.get( "id" ) );
    println!( "{:?}", cookies.get( "site" ) );

    NamedFile::open( index_path )
        .await
        .map_err( |_|  NotFound("Page not found".to_string()) )
}

// /guest/s/default/?ap=70:a7:41:dd:7a:78&id=4c:eb:42:9b:82:55&t=1734714029&url=http://www.msftconnecttest.com%2Fredirect&ssid=Wi-Fi_Visitantes%20
#[ get( "/s/<site>?<ap>&<id>&<t>&<url>&<ssid>" ) ]
pub async fn guest_register( cookies: &CookieJar<'_>, site: String, ap: String, id: String, t: String, url: String, ssid: String ) -> Result< Redirect, BadRequest<Json<Error>> > {
    let _ = url;

    cookies.add( ( "ap",     ap.clone() ) );
    cookies.add( ( "id",     id.clone() ) );
    cookies.add( ( "t",       t.clone() ) );
    cookies.add( ( "ssid", ssid.clone() ) );
    cookies.add( ( "site", site.clone() ) );

    Ok( Redirect::to( "/guest/index" ) )
}

#[ post( "/guest/connect",  format="application/json", data="<secret_code>", rank = 1 ) ]
pub async fn connect_guest_with_authenticator( cookies: &CookieJar<'_>, unifi: &State< Arc< Mutex<UnifiController> > > , secret_code: Json<SecretCode> ) -> Result< Accepted<()>, Custom< Json<Error> > > {
    let mut unifi = unifi.lock().await;

    let mac = cookies.get( "id" ).unwrap().value().to_string();
    let site = cookies.get( "site" ).unwrap().value().to_string();
    let minutes: u16 = 180;

    let res = unifi.authorize_guest( &site, &mac, &minutes ).await;

    Ok( Accepted( () ) )
}
