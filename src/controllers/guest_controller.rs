use chrono::Local;
use rocket::http::{CookieJar, Status};
use rocket::response::{
    Redirect,
    status::{BadRequest, Custom},
};
use rocket::serde::json::Json;
use rocket::{get, post};

use crate::model::entity::admin::Admin;
use crate::model::entity::guest::{Guest, GuestData, GuestStatus};
use crate::model::repository::Repository;
use crate::model::repository::approver_repository::ApproverRepository;
use crate::model::repository::guest_repository::GuestRepository;
use crate::security::approval_code::validate_code;
use crate::unifi::unifi::UnifiController;
use crate::utils::error::Error;

// ENDPOINTS
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

    Ok(Redirect::to("/guest/"))
}

#[get("/guest", format = "application/json")]
pub async fn get_guests(
    admin: Option<Admin>,
    guest_repo: GuestRepository,
) -> Result<Custom<Json<Vec<Guest>>>, Custom<Json<Error>>> {
    if admin.is_none() {
        return Err(Error::new_with_custom(
            "Unauthorized user",
            Local::now().to_string(),
            401,
        ));
    }

    let guests = guest_repo.find_all().await;
    Ok(Custom(Status::Ok, Json(guests)))
}

#[post("/guest/connect", format = "application/json", data = "<guest_data>")]
pub async fn guest_connection_request(
    cookies: &CookieJar<'_>,
    repository: GuestRepository,
    approver_repository: ApproverRepository,
    mut unifi: UnifiController,
    guest_data: Json<GuestData>,
    admin: Option<Admin>,
) -> Result<Status, Custom<Json<Error>>> {
    let guest_data = guest_data.into_inner();

    match guest_data {
        // Form Call
        GuestData::Form(guest_form) => {
            let mac = cookies.get("id").unwrap().value().to_string();
            let site = cookies.get("site").unwrap().value().to_string();
            let minutes: u16 = 180;

            let mut guest = Guest {
                id: String::new(),
                active: true,
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

            // Approval by code
            if let Some(code) = guest_form.au_code {
                let approver = validate_code(code, &approver_repository).await;
                if approver.is_none() {
                    return Err(Error::new_with_custom(
                        "Invalid Fields",
                        Local::now().to_string(),
                        400,
                    ));
                }

                guest.approver = approver.unwrap();
                let res = unifi.authorize_guest(&site, &mac, &minutes).await;
                match res {
                    Ok(_) => {
                        let _ = repository.save(guest).await;
                    }
                    Err(_) => {}
                }

                return Ok(Status::Accepted);
            }

            // Approval pending
            let _ = repository.save(guest).await;
            return Ok(Status::Ok);
        }

        // API Call
        GuestData::Info(guest_info) => {
            if admin.is_none() {
                return Err(Error::new_with_custom(
                    "Unauthorized user",
                    Local::now().to_string(),
                    401,
                ));
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

                        return Ok(Status::Ok);
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
                    let guest = Guest {
                        id: String::new(),
                        active: true,

                        full_name: String::from("---"),
                        email: String::from("---"),
                        phone: String::from("---"),
                        cpf: String::from("---"),

                        approver: admin.unwrap().name,
                        status: if guest_info.approved {
                            GuestStatus::Approved
                        } else {
                            GuestStatus::Reject
                        },
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
