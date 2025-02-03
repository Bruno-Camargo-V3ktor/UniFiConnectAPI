use std::env;

use chrono::Local;
use rocket::fs::NamedFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Route, get, post, put, routes};

use crate::model::entity::admin::Admin;
use crate::model::entity::client::{Client, ClientData, ClientInfo, ClientStatus};
use crate::model::repository::Repository;
use crate::model::repository::approver_repository::ApproverRepository;
use crate::model::repository::client_repository::ClientRepository;
use crate::security::approval_code::validate_code;
use crate::unifi::unifi::UnifiController;
use crate::utils::error::{BadRequest, CustomError, Error, Unauthorized};
use crate::utils::responses::{CustomStatus, Ok, Response};

// ENDPOINTS
#[get("/<_..>")]
pub async fn client_connect_page() -> Result<NamedFile, ()> {
    let mut path = env::var("STATIC_FILES_DIR").expect("STATIC_FILES_DIR NOT DEFINED");
    path.push_str("/client/index.html");

    Ok(NamedFile::open(path)
        .await
        .expect("Client Connect Page Not Found"))
}

#[get("/s/<site>?<ap>&<id>&<t>&<url>&<ssid>", format = "text/html")]
pub async fn client_register(
    cookies: &CookieJar<'_>,
    site: String,
    ap: String,
    id: String,
    t: String,
    url: String,
    ssid: String,
) -> Result<Redirect, ()> {
    // /client/s/default/?ap=70:a7:41:dd:7a:78&id=4c:eb:42:9b:82:55&t=1734714029&url=http://www.msftconnecttest.com%2Fredirect&ssid=Wi-Fi_Visitantes%20

    cookies.add(("ap", ap.clone()));
    cookies.add(("id", id.clone()));
    cookies.add(("t", t.clone()));
    cookies.add(("ssid", ssid.clone()));
    cookies.add(("site", site.clone()));
    cookies.add(("url", url.clone()));

    Ok(Redirect::to("/client/"))
}

#[post("/client/connect", format = "application/json", data = "<data>")]
pub async fn client_connection_api(
    mut unifi: UnifiController,
    repository: ClientRepository,
    data: Json<ClientInfo>,
    admin: Option<Admin>,
) -> Result<CustomStatus, CustomError> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }
    let client = data.into_inner();

    // Approving a pending order
    if let Some(id) = client.id.clone() {
        if let Some(mut c) = repository.find_by_id(id).await {
            if client.connect {
                c.approver = admin.unwrap().name;
                c.status = ClientStatus::Approved;
                c.start_time = Local::now();

                let _ = unifi
                    .authorize_device(&c.site, &c.mac, &client.minutes)
                    .await;
            } else {
                c.status = ClientStatus::Reject;
            }

            repository.update(c).await;

            return Ok(Response::new_custom_status(200));
        }
    }

    // Direct approval
    let res = if client.connect {
        unifi
            .authorize_device(&client.site, &client.mac, &client.minutes)
            .await
    } else {
        unifi.unauthorize_device(&client.site, &client.mac).await
    };

    match res {
        Ok(_) => {
            let mut new_client = Client::new_with_info(&client);
            new_client.approver = admin.unwrap().name;

            let _ = repository.save(new_client).await;
        }
        Err(_) => {}
    }

    Ok(Response::new_custom_status(200))
}

#[post("/client/connect?form", format = "application/json", data = "<data>")]
pub async fn client_connection_approver(
    mut unifi: UnifiController,
    cookies: &CookieJar<'_>,
    repository: ClientRepository,
    approver_repository: ApproverRepository,
    data: Json<ClientData>,
) -> Result<CustomStatus, BadRequest> {
    let client = data.into_inner();

    if !client.validate_form() {
        return Err(Error::new_bad_request("Invalid Form Field(s)"));
    }

    let mac = cookies.get("id").unwrap().value().to_string();
    let site = cookies.get("site").unwrap().value().to_string();
    let minutes: u16 = env::var("DEFAULT_APPROVAL_TIME")
        .unwrap_or("180".to_string())
        .parse()
        .expect("DEFAULT_APPROVAL_TIME NOT NUMBER");

    let mut new_client = Client::new_with_data(&client);
    new_client.site = site.clone();
    new_client.mac = mac.clone();
    new_client.time_connection = minutes.to_string();

    // Approval by code
    if let Some(code) = client.approver_code {
        let approver = validate_code(code, &approver_repository).await;
        if approver.is_none() {
            return Err(Error::new_bad_request("Invalid Fields"));
        }

        new_client.status = ClientStatus::Approved;
        new_client.approver = approver.unwrap();
        let res = unifi.authorize_device(&site, &mac, &minutes).await;
        match res {
            Ok(_) => {
                let _ = repository.save(new_client).await;
            }
            Err(_) => {}
        }

        return Ok(Response::new_custom_status(202));
    }

    // Approval pending
    let _ = repository.save(new_client).await;
    return Ok(Response::new_custom_status(200));
}

#[get("/client", format = "application/json")]
pub async fn get_clients(
    admin: Option<Admin>,
    client_repo: ClientRepository,
) -> Result<Ok<Vec<Client>>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let clients = client_repo.find_all().await;

    Ok(Response::new_ok(clients))
}

#[put("/client", format = "application/json", data = "<data>")]
pub async fn update_client(
    admin: Option<Admin>,
    client_repo: ClientRepository,
    data: Json<Client>,
) -> Result<Ok<()>, Unauthorized> {
    if admin.is_none() {
        return Err(Error::new_unauthorized("Unauthorized user"));
    }

    let _ = client_repo.update(data.into_inner()).await;
    Ok(Response::new_ok(()))
}

// Functions
pub fn routes() -> Vec<Route> {
    routes![
        client_connection_api,
        client_connection_approver,
        get_clients,
        update_client
    ]
}
