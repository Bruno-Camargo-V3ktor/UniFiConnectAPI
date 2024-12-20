use rocket::{ get };
use rocket::serde::json::Json;
use rocket::response::{ Redirect, status::{BadRequest, NotFound} };
use rocket::http::CookieJar;
use rocket::fs::NamedFile;

use crate::utils::error::Error;

use std::env;

// ENDPOINTS

#[ get( "/index/<_..>" ) ]
pub async fn guest_page( cookies: &CookieJar<'_> ) -> Result< NamedFile, NotFound<String> > {
    let index_path = env::var( "GUEST_LOGIN_PAGE" ).unwrap();

    println!( "{:?}", cookies.get( "id" ) );
    println!( "{:?}", cookies.get( "ap" ) );
    println!( "{:?}", cookies.get( "t" ) );
    println!( "{:?}", cookies.get( "ssid" ) );

    NamedFile::open( index_path )
        .await
        .map_err( |_|  NotFound("Page not found".to_string()) )

}

// /guest/s/default/?ap=70:a7:41:dd:7a:78&id=4c:eb:42:9b:82:55&t=1734714029&url=http://www.msftconnecttest.com%2Fredirect&ssid=Wi-Fi_Visitantes%20
#[ get( "/s/default?<ap>&<id>&<t>&<url>&<ssid>" ) ]
pub async fn guest_register( cookies: &CookieJar<'_>, ap: String, id: String, t: String, url: String, ssid: String ) -> Result< Redirect, BadRequest<Json<Error>> > {


   cookies.add( ( "ap",    ap.clone() ) );
   cookies.add( ( "id",    id.clone() ) );
   cookies.add( ( "t",      t.clone() ) );
   cookies.add( ( "ssid", ssid.clone() ) );

    Ok( Redirect::to( "/guest/index" ) )
}
