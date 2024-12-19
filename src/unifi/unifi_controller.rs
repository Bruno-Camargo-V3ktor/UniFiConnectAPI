use std::collections::HashMap;

// Structs
pub struct UnifiController {
    base_url: String,
    username: String,
    password: String,
    site: String,
    authentication: Option< (String, String) >
}


// Impls
impl UnifiController {

    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self{ base_url, username, password, site: "default".to_string(), authentication: None }
    }

    pub async fn authentication_api(&mut self) -> Result< (String, String), reqwest::Error> {
        let body = HashMap::from( [("username", self.username.as_str()), ("password", self.password.as_str())] );

        let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // Ignorar certificados inválidos
                .build().unwrap();

        let res = client.post( format!( "{}/login", self.base_url ) )
            .json(&body)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => {
                let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap().to_string();
                let cookie_vec = cookie_str.split_terminator( &['=', ';'] ).collect::< Vec<&str> >();
                let cookie = ( cookie_vec[0].to_string(), cookie_vec[0].to_string() );

                self.authentication = Some( cookie.clone() );
                Ok( cookie )
            }

            _ => { Ok( ("".to_string(), "".to_string()) )  }
        }

    }

}
