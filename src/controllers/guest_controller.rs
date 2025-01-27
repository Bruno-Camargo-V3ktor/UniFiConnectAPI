use std::env;

use chrono::Local;
use rocket::fs::NamedFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Route, get, post, routes};

use crate::model::entity::admin::Admin;
use crate::model::entity::guest::{Guest, GuestData, GuestStatus};
use crate::model::repository::Repository;
use crate::model::repository::approver_repository::ApproverRepository;
use crate::model::repository::guest_repository::GuestRepository;
use crate::security::approval_code::validate_code;
use crate::unifi::unifi::UnifiController;
use crate::utils::error::{CustomError, Error, Unauthorized};
use crate::utils::responses::{CustomStatus, Ok, Response};

// ENDPOINTS
#[get("/<_..>")]
pub async fn guest_page() -> Result<NamedFile, ()> {
    let mut path = env::var("STATIC_FILES_DIR").expect("STATIC_FILES_DIR NOT DEFINED");
    path.push_str("/guest/index.html");

    Ok(NamedFile::open(path).await.expect("Guest Page Not Found"))
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
) -> Result<Redirect, ()> {
    // /guest/s/default/?ap=70:a7:41:dd:7a:78&id=4c:eb:42:9b:82:55&t=1734714029&url=http://www.msftconnecttest.com%2Fredirect&ssid=Wi-Fi_Visitantes%20

    cookies.add(("ap", ap.clone()));
    cookies.add(("id", id.clone()));
    cookies.add(("t", t.clone()));
    cookies.add(("ssid", ssid.clone()));
    cookies.add(("site", site.clone()));
    cookies.add(("url", url.clone()));

    Ok(Redirect::to("/guest/"))
}

#[post("/guest/connect", format = "application/json", data = "<guest_data>")]
pub async fn guest_connection_request(
    cookies: &CookieJar<'_>,
    repository: GuestRepository,
    approver_repository: ApproverRepository,
    mut unifi: UnifiController,
    guest_data: Json<GuestData>,
    admin: Option<Admin>,
) -> Result<CustomStatus, CustomError> {
    let guest_data = guest_data.into_inner();

    match guest_data {
        // Form Call
        GuestData::Form(guest_form) => {
            if !guest_form.validate_form() {
                return Err(Error::new_bad_request("Invalid Form Field(s)"));
            }

            let mac = cookies.get("id").unwrap().value().to_string();
            let site = cookies.get("site").unwrap().value().to_string();
            let minutes: u16 = 180;

            let mut guest = Guest::new();
            guest.full_name = guest_form.full_name;
            guest.email = guest_form.email;
            guest.phone = guest_form.phone;
            guest.cpf = guest_form.cpf;
            guest.site = site.clone();
            guest.mac = mac.clone();
            guest.time_connection = minutes.to_string();

            // Approval by code
            if let Some(code) = guest_form.au_code {
                let approver = validate_code(code, &approver_repository).await;
                if approver.is_none() {
                    return Err(Error::new_bad_request("Invalid Fields"));
                }

                guest.status = GuestStatus::Approved;
                guest.approver = approver.unwrap();
                let res = unifi.authorize_guest(&site, &mac, &minutes).await;
                match res {
                    Ok(_) => {
                        let _ = repository.save(guest).await;
                    }
                    Err(_) => {}
                }

                return Ok(Response::new_custom_status(202));
            }

            // Approval pending
            let _ = repository.save(guest).await;
            return Ok(Response::new_custom_status(200));
        }

        // API Call
        GuestData::Info(guest_info) => {
            if admin.is_none() {
                return Err(Error::new_unauthorized("Unauthorized user"));
            }

            // Approving a pending order
            match guest_info.id {
                Some(id) => {
                    if let Some(mut g) = repository.find_by_id(id).await {
                        if guest_info.approved {
                            g.approver = admin.unwrap().name;
                            g.status = GuestStatus::Approved;
                            g.start_time = Local::now();

                            let _ = unifi
                                .authorize_guest(&g.site, &g.mac, &guest_info.minutes)
                                .await;
                        } else {
                            g.status = GuestStatus::Reject;
                        }

                        repository.update(g).await;

                        return Ok(Response::new_custom_status(200));
                    }
                }

                _ => {}
            }

            // Direct approval
            let res = if guest_info.approved {
                unifi
                    .authorize_guest(&guest_info.site, &guest_info.mac, &guest_info.minutes)
                    .await
            } else {
                unifi
                    .unauthorize_guest(&guest_info.site, &guest_info.mac)
                    .await
            };

            match res {
                Ok(_) => {
                    let mut guest = Guest::new();
                    guest.mac = guest_info.mac;
                    guest.site = guest_info.site;
                    guest.approver = admin.unwrap().name;
                    guest.time_connection = guest_info.minutes.to_string();
                    guest.status = if guest_info.approved {
                        GuestStatus::Approved
                    } else {
                        GuestStatus::Reject
                    };

                    let _ = repository.save(guest).await;
                }
                Err(_) => {}
            }

            Ok(Response::new_custom_status(200))
        }
    }
}

#[get("/guest", format = "application/json")]
pub async fn get_guests(
    admin: Option<Admin>,
    guest_repo: GuestRepository,
) -> Result<Ok<Vec<Guest>>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let guests = guest_repo.find_all().await;

    Ok(Response::new_ok(guests))
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![guest_connection_request, get_guests]
}
