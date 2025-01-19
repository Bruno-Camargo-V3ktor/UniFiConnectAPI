use chrono::Local;
use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::{
    Redirect,
    status::{BadRequest, Custom, NotFound},
};
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;

use crate::db::mongo_db::MongoDb;
use crate::model::entity::guest::{Guest, GuestData, GuestStatus};
use crate::model::repository::Repository;
use crate::model::repository::guest_repository::GuestRepository;
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

    cookies.add(("ap", ap.clone()));
    cookies.add(("id", id.clone()));
    cookies.add(("t", t.clone()));
    cookies.add(("ssid", ssid.clone()));
    cookies.add(("site", site.clone()));
    cookies.add(("url", url.clone()));

    Ok(Redirect::to("/guest/index"))
}

#[post("/guest/connect", format = "application/json", data = "<guest_data>")]
pub async fn guest_connection_request(
    cookies: &CookieJar<'_>,
    db: Connection<MongoDb>,
    unifi: &UnifiState,
    guest_data: Json<GuestData>,
) -> Result<Status, Custom<Json<Error>>> {
    let repository = GuestRepository {
        database: db.default_database().unwrap(),
        name: String::from("Guest"),
    };

    let guest_data = guest_data.into_inner();

    match guest_data {
        // Form Call
        GuestData::Form(guest_form) => {
            let mac = cookies.get("id").unwrap().value().to_string();
            let site = cookies.get("site").unwrap().value().to_string();
            let minutes: u16 = 180;

            let guest = Guest {
                id: String::new(),
                full_name: guest_form.full_name,
                email: guest_form.email,
                phone: guest_form.phone,
                cpf: guest_form.cpf,
                site: site.clone(),
                approver: "---".to_string(),
                status: GuestStatus::Pending,
                mac: mac.clone(),
                time_connection: minutes.to_string(),
                start_time: Local::now(),
            };

            if let Some(code) = guest_form.au_code {
                let mut unifi = unifi.lock().await;

                let res = unifi.authorize_guest(&site, &mac, &minutes).await;
                match res {
                    Ok(_) => {
                        let _ = repository.save(guest).await;
                    }
                    Err(_) => {}
                }

                return Ok(Status::Accepted);
            }

            return Ok(Status::Ok);
        }

        // API Call
        GuestData::Info(guest_info) => {
            let mut unifi = unifi.lock().await;

            match guest_info.id {
                Some(id) => {
                    if let Some(mut g) = repository.find_by_id(id).await {
                        if guest_info.approved {
                            g.approver = String::from("---");
                            g.status = GuestStatus::Approved;
                            g.start_time = Local::now();

                            let _ = unifi
                                .authorize_guest(&g.site, &g.mac, &guest_info.minutes)
                                .await;
                        } else {
                            g.status = GuestStatus::Reject;
                        }

                        repository.update(g).await;

                        return Ok(Status::Ok);
                    }
                }

                None => {}
            }

            let res = unifi
                .authorize_guest(&guest_info.site, &guest_info.mac, &guest_info.minutes)
                .await;

            match res {
                Ok(_) => {
                    let guest = Guest {
                        id: String::new(),
                        full_name: String::from("---"),
                        email: String::from("---"),
                        phone: String::from("---"),
                        cpf: String::from("---"),

                        approver: "---".to_string(),
                        status: GuestStatus::Approved,
                        mac: guest_info.mac,
                        site: guest_info.site,

                        time_connection: guest_info.minutes.to_string(),
                        start_time: Local::now(),
                    };

                    let _ = repository.save(guest).await;
                }
                Err(_) => {}
            }

            Ok(Status::Ok)
        }
    }
}
