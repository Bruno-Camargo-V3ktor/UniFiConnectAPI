use rocket::State;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Types
pub type UnifiState = State<Arc<Mutex<UnifiController>>>;

// Structs
#[derive(Clone)]
pub struct UnifiController {
    base_url: String,
    username: String,
    password: String,
    client: reqwest::Client,
    authentication_time: Option<Instant>,
}

#[derive(Serialize, Deserialize)]
pub struct GuestAuthorization {
    cmd: String,
    mac: Option<String>,
    minutes: Option<u16>,
}

#[derive(Serialize, Deserialize)]
pub struct GuestUnauthorize {
    cmd: String,
    mac: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientInfo {
    /// ID interno do UniFi para este registro
    #[serde(rename = "_id")]
    pub record_id: Option<String>,

    /// Outro ID interno que às vezes aparece (ex.: "user_id")
    #[serde(rename = "user_id")]
    pub user_id: Option<String>,

    /// MAC address do dispositivo convidado
    pub mac: String,

    /// Hostname do dispositivo
    pub hostname: Option<String>,

    /// Indica se o cliente está ou não autorizado no portal
    pub authorized: Option<bool>,

    /// True se for tratado como convidado
    #[serde(rename = "is_guest")]
    pub is_guest: Option<bool>,

    /// Momento (em unix epoch) em que foi associada a rede
    pub assoc_time: Option<u64>,

    /// Timestamp mais recente em que o controller viu atividade desse convidado
    pub last_seen: Option<u64>,

    /// Início da autorização (caso exista, p.ex. para voucher)
    pub start: Option<u64>,

    /// Fim da autorização (caso exista, se há tempo de expiração)
    pub end: Option<u64>,

    /// IP obtido via DHCP (se conectado)
    pub ip: Option<String>,

    /// Nome do SSID guest
    pub essid: Option<String>,

    /// Nome do AP ao qual este cliente está/estava conectado
    #[serde(rename = "ap_name")]
    pub ap_name: Option<String>,

    /// MAC do Access Point
    #[serde(rename = "ap_mac")]
    pub ap_mac: Option<String>,

    /// Exemplo de campo que o UniFi usa para controle de vouchers
    #[serde(rename = "voucher_code")]
    pub voucher_code: Option<String>,

    /// Total de bytes consumidos nesta sessão (upload + download)
    pub bytes: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    meta: Value,
    data: Value,
}

// Impls
impl GuestAuthorization {
    pub fn new(mac: String, minutes: u16) -> Self {
        Self {
            cmd: String::from("authorize-guest"),
            mac: Some(mac),
            minutes: Some(minutes),
        }
    }
}

impl GuestUnauthorize {
    pub fn new(mac: String) -> Self {
        Self {
            cmd: String::from("unauthorize-guest"),
            mac: Some(mac),
        }
    }
}

impl UnifiController {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            base_url,
            username,
            password,
            authentication_time: None,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // Ignorar certificados inválidos
                .cookie_store(true) // Habilitando o armazenamento e envio automatico de cookies
                .build()
                .unwrap(),
        }
    }

    fn check_authentication(&mut self) -> bool {
        match self.authentication_time {
            Some(t) => {
                let duration = t.elapsed();
                if duration <= Duration::from_secs(60 * 10) {
                    return true;
                }

                self.authentication_time = None;
                false
            }
            None => false,
        }
    }

    pub async fn authentication_api(&mut self) -> Result<(), reqwest::Error> {
        let body = HashMap::from([
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
        ]);

        let res = self
            .client
            .post(format!("{}/login", self.base_url))
            .json(&body)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => {
                self.authentication_time = Some(Instant::now());
                Ok(())
            }

            _ => Ok(()),
        }
    }

    pub async fn authorize_guest(
        &mut self,
        site: &String,
        mac: &String,
        minutes: &u16,
    ) -> Result<(), reqwest::Error> {
        if !self.check_authentication() {
            let _ = self.authentication_api().await?;
        }

        let body = GuestAuthorization::new(mac.to_string(), *minutes);

        let _res = self
            .client
            .post(format!("{}/s/{}/cmd/stamgr", self.base_url, site))
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn unauthorize_guest(
        &mut self,
        site: &String,
        mac: &String,
    ) -> Result<(), reqwest::Error> {
        if !self.check_authentication() {
            let _ = self.authentication_api().await?;
        }

        let body = GuestUnauthorize::new(mac.to_string());

        let _res = self
            .client
            .post(format!("{}/s/{}/cmd/stamgr", self.base_url, site))
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn rename_device_client(
        &mut self,
        id: String,
        site: String,
        name: String,
    ) -> Result<(), reqwest::Error> {
        if !self.check_authentication() {
            let _ = self.authentication_api().await?;
        }

        let body = HashMap::from([("name", name.as_str())]);
        let _res = self
            .client
            .put(format!("{}/s/{}/upd/user/{}", self.base_url, site, id))
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_guest_clients(&self, site: String) -> Result<Vec<ClientInfo>, reqwest::Error> {
        let res = self
            .client
            .get(format!("{}/s/{}/stat/guest", self.base_url, site))
            .send()
            .await?;

        let res = res.json::<ApiResponse>().await?;
        let mut list: Vec<ClientInfo> = vec![];

        match res.data {
            Value::Array(array) => {
                let clients: Result<Vec<ClientInfo>, _> =
                    serde_json::from_value(Value::Array(array));

                if let Ok(cs) = clients {
                    list = cs
                }
            }
            _ => {}
        }

        Ok(list)
    }
}

// Guards
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UnifiController {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let unifi = request.guard::<&UnifiState>().await.unwrap().lock().await;
        Outcome::Success(unifi.clone())
    }
}
