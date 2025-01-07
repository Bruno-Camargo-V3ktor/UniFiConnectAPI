use std::collections::HashMap;
use std::time::{Instant, Duration};
use rocket::serde::{Deserialize, Serialize};

// Structs
pub struct UnifiController {
    base_url: String,
    username: String,
    password: String,
    client: reqwest::Client,
    authentication_time: Option<Instant>
}

#[ derive( Serialize, Deserialize ) ]
pub struct GuestAuthorization {
    cmd: String,
    mac: Option<String>,
    minutes: Option<u16>
}

// Impls
impl GuestAuthorization {
    pub fn new(mac: String, minutes: u16) -> Self {
        Self{ cmd: String::from( "authorize-guest" ), mac: Some(mac), minutes: Some(minutes) }
    }
}


impl UnifiController {

    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self{
            base_url,
            username,
            password,
            authentication_time: None,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // Ignorar certificados inválidos
                .cookie_store(true) // Habilitando o armazenamento e envio automatico de cookies
                .build().unwrap(),
        }
    }

    fn check_authentication(&mut self) -> bool {
        match self.authentication_time {
            Some( t ) => {
                let duration = t.elapsed();
                if duration <= Duration::from_secs(60 * 10)  {
                    return true;
                }

                self.authentication_time = None;
                false
            }
            None => false
        }
    }

    pub async fn authentication_api(&mut self) -> Result< (), reqwest::Error> {
        let body = HashMap::from( [("username", self.username.as_str()), ("password", self.password.as_str())] );

        let res = self.client.post( format!( "{}/login", self.base_url ) )
            .json(&body)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => {
                self.authentication_time = Some( Instant::now() );
                Ok( () )
            }

            _ => { Ok( () )  }
        }

    }

    pub async fn authorize_guest(&mut self, site: &String, mac: &String, minutes: &u16) -> Result< (), reqwest::Error > {
        if !self.check_authentication() { let _ = self.authentication_api().await?; }

        let body = GuestAuthorization::new( mac.to_string(), *minutes );

        let _res = self.client.post( format!( "{}/s/{}/cmd/stamgr", self.base_url, site ) )
            .json( &body )
            .send()
            .await?;

        Ok( () )
    }

}
