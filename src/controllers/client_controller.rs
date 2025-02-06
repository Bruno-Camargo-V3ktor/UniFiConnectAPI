use crate::configurations::config::ConfigApp;
use crate::model::entity::admin::Admin;
use crate::model::entity::approver::Approver;
use crate::model::entity::client::{Client, ClientData, ClientInfo, ClientStatus};
use crate::model::repository::Repository;
use crate::model::repository::mongo_repository::MongoRepository;
use crate::security::approval_code::validate_code;
use crate::unifi::unifi::UnifiController;
use crate::utils::error::{BadRequest, CustomError, Error, Unauthorized};
use crate::utils::responses::{CustomStatus, Ok, Response};
use chrono::Local;
use rocket::fs::NamedFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{Route, State, get, post, put, routes};

// ENDPOINTS
#[get("/<_..>")]
pub async fn client_connect_page(config: &State<ConfigApp>) -> Result<NamedFile, ()> {
    let config = config.read().await;
    let mut path = config.server.files_dir.clone();
    path.push_str("/client/index.html");

    Ok(NamedFile::open(path)
        .await
        .expect("Client Connect Page Not Found"))
}

#[get("/guest/s/<site>?<ap>&<id>&<t>&<url>&<ssid>", format = "text/html")]
pub async fn client_register(
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

    Ok(Redirect::to("/client/"))
}

#[post("/client/connect", format = "application/json", data = "<data>")]
pub async fn client_connection_api(
    mut unifi: UnifiController,
    repository: MongoRepository<Client>,
    data: Json<ClientInfo>,
    admin: Admin,
    config: &State<ConfigApp>,
) -> Result<CustomStatus, CustomError> {
    let config = config.read().await;
    let client = data.into_inner();

    // Approving a pending order
    if let Some(id) = client.id.clone() {
        if let Some(mut c) = repository.find_by_id(id).await {
            if client.connect {
                let group = {
                    if let Some(g) = config.clients.find_group(&c.client_type) {
                        g
                    } else {
                        return Err(Error::new_bad_request("Invalid Fields"));
                    }
                };

                c.approver = admin.name;
                c.status = ClientStatus::Approved;
                c.start_time = Local::now();

                let _ = unifi.conect_client(&c, &group).await;
            } else {
                c.status = ClientStatus::Reject;
            }

            repository.update(c).await;

            return Ok(Response::new_custom_status(200));
        }
    }

    // Direct approval
    let mut new_client = Client::new_with_info(&client);
    new_client.approver = admin.name;

    let group = if let Some(g) = config.clients.find_group(&new_client.client_type) {
        g
    } else {
        return Err(Error::new_bad_request("Invalid Fields"));
    };

    if client.connect {
        unifi.conect_client(&new_client, &group).await;
        let _ = repository.save(new_client).await;
    } else {
        let _ = unifi.unauthorize_device(&client.site, &client.mac).await;
    };

    Ok(Response::new_custom_status(200))
}

#[post("/client/connect?form", format = "application/json", data = "<data>")]
pub async fn client_connection_approver(
    mut unifi: UnifiController,
    cookies: &CookieJar<'_>,
    repository: MongoRepository<Client>,
    approver_repository: MongoRepository<Approver>,
    data: Json<ClientData>,
    config: &State<ConfigApp>,
) -> Result<CustomStatus, BadRequest> {
    let config = config.read().await;
    let client = data.into_inner();

    if !client.validate_form() {
        return Err(Error::new_bad_request("Invalid Form Field(s)"));
    }

    let group = {
        if let Some(g) = config.clients.find_group(&client.client_type) {
            g
        } else {
            return Err(Error::new_bad_request("Invalid Fields"));
        }
    };

    let mac = cookies.get("id").unwrap().value().to_string();
    let site = cookies.get("site").unwrap().value().to_string();
    let minutes: u16 = group.time_conneciton.clone() as u16;

    let mut new_client = Client::new_with_data(&client);
    new_client.site = site.clone();
    new_client.mac = mac.clone();
    new_client.time_connection = minutes.to_string();

    // Approval by code
    if let Some(code) = client.approver_code {
        let approver = validate_code(code, &client.client_type, &approver_repository).await;
        if approver.is_none() {
            return Err(Error::new_bad_request("Invalid Fields"));
        }

        new_client.status = ClientStatus::Approved;
        new_client.approver = approver.unwrap();

        unifi.conect_client(&new_client, &group).await;
        let _ = repository.save(new_client).await;

        return Ok(Response::new_custom_status(202));
    }

    // Approval pending
    let _ = repository.save(new_client).await;
    return Ok(Response::new_custom_status(200));
}

#[get("/client", format = "application/json")]
pub async fn get_clients(
    _admin: Admin,
    client_repo: MongoRepository<Client>,
) -> Result<Ok<Vec<Client>>, Unauthorized> {
    let clients = client_repo.find_all().await;

    Ok(Response::new_ok(clients))
}

#[put("/client", format = "application/json", data = "<data>")]
pub async fn update_client(
    _admin: Admin,
    client_repo: MongoRepository<Client>,
    data: Json<Client>,
) -> Result<Ok<()>, Unauthorized> {
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
