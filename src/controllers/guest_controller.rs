use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::{
    Redirect,
    status::{BadRequest, Custom, NotFound},
};
use rocket::serde::json::Json;
use rocket::{get, post};

use crate::model::entity::guest::GuestData;
use crate::unifi::unifi::UnifiState;
use crate::utils::error::Error;

use std::env;

// ENDPOINTS
#[get("/index/<_..>", format = "text/html")]
pub async fn guest_page() -> Result<NamedFile, NotFound<String>> {
    let index_path = env::var("GUEST_LOGIN_PAGE").unwrap();

    NamedFile::open(index_path)
        .await
        .map_err(|_| NotFound("Page not found".to_string()))
}

#[get("/s/<site>?<ap>&<id>&<t>&<url>&<ssid>", format = "text/html")]
pub async fn guest_register(
    cookies: &CookieJar<'_>,
    site: String,
    ap: String,
    id: String,
    t: String,
    url: String,
    ssid: String,
) -> Result<Redirect, BadRequest<Json<Error>>> {
    // /guest/s/default/?ap=70:a7:41:dd:7a:78&id=4c:eb:42:9b:82:55&t=1734714029&url=http://www.msftconnecttest.com%2Fredirect&ssid=Wi-Fi_Visitantes%20
    let _ = url;

    cookies.add(("ap", ap.clone()));
    cookies.add(("id", id.clone()));
    cookies.add(("t", t.clone()));
    cookies.add(("ssid", ssid.clone()));
    cookies.add(("site", site.clone()));

    Ok(Redirect::to("/guest/index"))
}

#[post("/guest/connect", format = "application/json", data = "<guest_data>")]
pub async fn guest_connection_request(
    cookies: &CookieJar<'_>,
    unifi: &UnifiState,
    guest_data: Json<GuestData>,
) -> Result<Status, Custom<Json<Error>>> {
    let guest_data = guest_data.into_inner();

    match guest_data {
        // Form Call
        GuestData::Form(guest_form) => {
            if let Some(code) = guest_form.au_code {
                let mut unifi = unifi.lock().await;

                let mac = cookies.get("id").unwrap().value().to_string();
                let site = cookies.get("site").unwrap().value().to_string();
                let minutes: u16 = 180;

                let res = unifi.authorize_guest(&site, &mac, &minutes).await;

                return Ok(Status::Accepted);
            }

            return Ok(Status::Ok);
        }

        // API Call
        GuestData::Info(guest_info) => {
            let mut unifi = unifi.lock().await;

            let r = unifi
                .authorize_guest(&guest_info.site, &guest_info.mac, &guest_info.minutes)
                .await;

            Ok(Status::Accepted)
        }
    }
}
